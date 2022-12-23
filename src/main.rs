#![deny(clippy::pedantic)]

use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::{fmt, iter};

use anyhow::Context;
use serde::{de, Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MatrixValue {
    String(String),
    List(Vec<MatrixValue>),
    Matrix(Matrix),
}

impl IntoIterator for MatrixValue {
    type Item = Box<dyn Iterator<Item = (String, String)>>;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::String(v) => {
                let it = iter::once((String::default(), v));
                let it: Self::Item = Box::new(it);
                Box::new(iter::once(it))
            }
            Self::List(v) => {
                let it = v.into_iter();
                let it = it.map(|v| -> Self::Item { Box::new(v.into_iter().flatten()) });
                Box::new(it)
            }
            Self::Matrix(v) => {
                let it = v.into_iter();
                Box::new(it)
            }
        }
    }
}

#[derive(Debug, Default)]
struct Matrix(Vec<(String, MatrixValue)>);

impl IntoIterator for Matrix {
    type Item = Box<dyn Iterator<Item = (String, String)>>;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let it = self.0.into_iter();
        let it = it.map(|(k, v)| -> Box<dyn Iterator<Item = _>> {
            let it = v.into_iter();
            Box::new(it.map(move |it| -> Self::Item {
                let k = k.clone();
                Box::new(it.map(move |(p, v)| {
                    let k = if p.is_empty() {
                        k.clone()
                    } else {
                        format!("{k}.{p}")
                    };
                    (k, v)
                }))
            }))
        });
        if let Some(it) = it.reduce(
            |l, r| -> Box<dyn Iterator<Item = Box<dyn Iterator<Item = (String, String)>>>> {
                let r: Vec<Vec<(String, String)>> = r.map(FromIterator::from_iter).collect();
                let it = l.into_iter().flat_map(move |l| {
                    let l: Vec<_> = l.collect();
                    let it = r
                        .clone()
                        .into_iter()
                        .map(move |r| -> Box<dyn Iterator<Item = _>> {
                            let it = l.clone().into_iter().chain(r);
                            Box::new(it)
                        });
                    Box::new(it)
                });
                Box::new(it)
            },
        ) {
            Box::new(it)
        } else {
            Box::new(iter::empty())
        }
    }
}

fn visit_ordered_map<'de, M, K, V>(mut access: M) -> Result<Vec<(K, V)>, M::Error>
where
    M: de::MapAccess<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    let mut m = Vec::with_capacity(access.size_hint().unwrap_or(0));
    while let Some(entry) = access.next_entry::<K, V>()? {
        m.push(entry);
    }
    Ok(m)
}

impl<'de> Deserialize<'de> for Matrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Matrix;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("workflow matrix")
            }

            fn visit_map<M>(self, access: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                visit_ordered_map(access).map(Matrix)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl From<Vec<(String, MatrixValue)>> for Matrix {
    fn from(matrix: Vec<(String, MatrixValue)>) -> Self {
        Self(matrix)
    }
}

impl From<Matrix> for MatrixValue {
    fn from(matrix: Matrix) -> Self {
        Self::Matrix(matrix)
    }
}

impl From<Vec<MatrixValue>> for MatrixValue {
    fn from(list: Vec<MatrixValue>) -> Self {
        Self::List(list)
    }
}

impl From<String> for MatrixValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for MatrixValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl Deref for Matrix {
    type Target = Vec<(String, MatrixValue)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
struct Strategy {
    pub matrix: Matrix,
}

#[derive(Clone, Debug, Deserialize)]
struct Template(String);

impl Template {
    fn apply(&self, matrix: &HashMap<String, String>) -> anyhow::Result<String> {
        let mut it = self.0.split("${{");
        let head = it.next().context("`name` missing")?;
        it.try_fold(head.into(), |head, part| {
            let (name, tail) = part.split_once("}}").context("failed to find `}}`")?;
            let name = name.trim();
            let name = name
                .strip_prefix("matrix.")
                .context(format!("missing `matrix.` prefix in `{name}`"))?;
            Ok(if let Some(v) = matrix.get(name) {
                format!("{head}{v}{tail}")
            } else {
                format!("{head}{tail}")
            }
            .into())
        })
        .map(|s: String| s.trim().replace("  ", " "))
    }
}

#[derive(Debug, Deserialize)]
struct Job {
    pub name: Option<Template>,
    pub strategy: Option<Strategy>,
}

fn deserialize_jobs<'de, D>(deserializer: D) -> Result<Vec<(String, Job)>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Vec<(String, Job)>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("job map")
        }

        fn visit_map<M>(self, access: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            visit_ordered_map(access)
        }
    }

    deserializer.deserialize_map(Visitor)
}

#[derive(Debug, Deserialize)]
struct Workflow {
    #[serde(deserialize_with = "deserialize_jobs")]
    pub jobs: Vec<(String, Job)>,
}

fn read_names(r: impl Read) -> anyhow::Result<Vec<String>> {
    let Workflow { jobs } = serde_yaml::from_reader(r).context("failed to parse yaml")?;
    let n = jobs.len();
    jobs.into_iter()
        .try_fold(Vec::with_capacity(n), |mut names, (name, job)| {
            match (job.name, job.strategy.map(|Strategy { matrix }| matrix)) {
                (None, None) => {
                    names.push(name);
                }
                (None, Some(matrix)) => {
                    let it = matrix
                        .into_iter()
                        .map(|e| e.map(|(_, v)| v).collect::<Vec<_>>().join(", "))
                        .map(|e| format!("{name} ({e})"));
                    names.extend(it);
                }

                (Some(name), None) => {
                    let name = name.apply(&HashMap::default())?;
                    names.push(name);
                }
                (Some(ref name), Some(matrix)) => {
                    let it = matrix.into_iter().flat_map(|it| name.apply(&it.collect()));
                    names.extend(it);
                }
            }
            Ok(names)
        })
}

fn main() -> anyhow::Result<()> {
    for path in args().skip(1) {
        let file = File::open(&path).context(format!("failed to open `{path}`"))?;
        for name in read_names(file).context("failed to parse job names")? {
            println!("{name}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_value_iter() {
        assert_eq!(
            MatrixValue::List(vec![
                MatrixValue::Matrix(
                    vec![
                        ("host".into(), "macos-latest".into()),
                        ("system".into(), "x86_64-darwin".into()),
                        ("check".into(), "clippy".into()),
                    ]
                    .into()
                ),
                MatrixValue::Matrix(vec![("foo".into(), "bar".into()),].into()),
            ])
            .into_iter()
            .map(Vec::from_iter)
            .collect::<Vec<_>>(),
            vec![
                vec![
                    ("host".into(), "macos-latest".into()),
                    ("system".into(), "x86_64-darwin".into()),
                    ("check".into(), "clippy".into()),
                ],
                vec![("foo".into(), "bar".into())]
            ]
        );
    }

    #[test]
    fn matrix_iter() {
        assert_eq!(
            Matrix(vec![(
                "config".into(),
                vec![
                    MatrixValue::Matrix(
                        vec![
                            ("host".into(), "macos-latest".into()),
                            ("system".into(), "x86_64-darwin".into()),
                            ("check".into(), "clippy".into()),
                        ]
                        .into()
                    ),
                    MatrixValue::Matrix(
                        vec![
                            ("host".into(), "macos-latest".into()),
                            ("system".into(), "x86_64-darwin".into()),
                            ("check".into(), "nextest".into()),
                        ]
                        .into()
                    ),
                    MatrixValue::Matrix(
                        vec![
                            ("host".into(), "ubuntu-latest".into()),
                            ("system".into(), "x86_64-linux".into()),
                            ("check".into(), "clippy".into()),
                        ]
                        .into()
                    ),
                    MatrixValue::Matrix(
                        vec![
                            ("host".into(), "ubuntu-latest".into()),
                            ("system".into(), "x86_64-linux".into()),
                            ("check".into(), "nextest".into()),
                        ]
                        .into()
                    ),
                    MatrixValue::Matrix(
                        vec![
                            ("host".into(), "ubuntu-latest".into()),
                            ("system".into(), "x86_64-linux".into()),
                            ("check".into(), "fmt".into()),
                        ]
                        .into()
                    ),
                ]
                .into()
            )])
            .into_iter()
            .map(Vec::from_iter)
            .collect::<Vec<_>>(),
            vec![
                vec![
                    ("config.host".into(), "macos-latest".into()),
                    ("config.system".into(), "x86_64-darwin".into()),
                    ("config.check".into(), "clippy".into()),
                ],
                vec![
                    ("config.host".into(), "macos-latest".into()),
                    ("config.system".into(), "x86_64-darwin".into()),
                    ("config.check".into(), "nextest".into()),
                ],
                vec![
                    ("config.host".into(), "ubuntu-latest".into()),
                    ("config.system".into(), "x86_64-linux".into()),
                    ("config.check".into(), "clippy".into()),
                ],
                vec![
                    ("config.host".into(), "ubuntu-latest".into()),
                    ("config.system".into(), "x86_64-linux".into()),
                    ("config.check".into(), "nextest".into()),
                ],
                vec![
                    ("config.host".into(), "ubuntu-latest".into()),
                    ("config.system".into(), "x86_64-linux".into()),
                    ("config.check".into(), "fmt".into()),
                ],
            ]
        );
    }

    #[test]
    fn check_names() {
        assert_eq!(
            read_names(include_str!("../testdata/check.yml").as_bytes()).unwrap(),
            vec![
                "nix fmt",
                "checks (macos-latest, x86_64-darwin, clippy)",
                "checks (macos-latest, x86_64-darwin, nextest)",
                "checks (ubuntu-latest, x86_64-linux, clippy)",
                "checks (ubuntu-latest, x86_64-linux, nextest)",
                "checks (ubuntu-latest, x86_64-linux, fmt)",
            ],
        );
    }

    #[test]
    fn lint_names() {
        assert_eq!(
            read_names(include_str!("../testdata/lint.yml").as_bytes()).unwrap(),
            vec![
                "cargo fmt",
                "cargo clippy (--workspace --all-targets)",
                "cargo clippy (--target=x86_64-unknown-linux-musl --workspace --all-targets)",
                "cargo clippy (--target=x86_64-unknown-none -p enarx-shim-sgx -p enarx-shim-kvm -p sallyport -p enarx_syscall_tests)",
                "cargo clippy (--target=wasm32-wasi -p enarx_wasm_tests --all-targets)",
                "cargo deny",
                "check-spdx-headers",
            ]
        );
    }

    #[test]
    fn test_names() {
        assert_eq!(
            read_names(include_str!("../testdata/test.yml").as_bytes()).unwrap(),
            vec![
                "enarx sev nightly debug",
                "enarx sev nightly debug with dbg",
                "enarx sev nightly release",
                "enarx sgx nightly debug",
                "enarx sgx nightly debug with dbg",
                "enarx sgx nightly release",
                "enarx kvm nightly debug",
                "enarx kvm nightly debug with dbg",
                "enarx kvm nightly release",
                "enarx build-only nightly default-features",
                "enarx build-only nightly gdb",
                "enarx MacOS",
                "enarx Windows",
                "nightly debug",
                "nightly release",
                "sallyport miri debug",
                "sallyport miri release"
            ]
        );
    }

    #[test]
    fn missing_names() {
        assert_eq!(
            read_names(include_str!("../testdata/missing.yml").as_bytes()).unwrap(),
            vec!["a b c"]
        );
    }
}
