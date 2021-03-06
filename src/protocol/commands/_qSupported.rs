use alloc::vec::Vec;

#[derive(PartialEq, Eq, Debug)]
pub struct qSupported<'a>(pub Vec<Feature<'a>>);

impl<'a> qSupported<'a> {
    pub fn parse(body: &'a str) -> Result<Self, ()> {
        if body.is_empty() {
            return Err(());
        }

        let features = body
            .split(';')
            .map(|s| match s.as_bytes().last() {
                None => {
                    // packet shouldn't have two ";;" in a row
                    Err(())
                }
                Some(&c) if c == b'+' || c == b'-' || c == b'?' => Ok(Feature {
                    name: &s[..s.len() - 1],
                    val: None,
                    status: match c {
                        b'+' => FeatureSupported::Yes,
                        b'-' => FeatureSupported::No,
                        b'?' => FeatureSupported::Maybe,
                        _ => unreachable!(),
                    },
                }),
                Some(_) => {
                    let mut parts = s.split('=');
                    Ok(Feature {
                        name: parts.next().unwrap(),
                        val: Some(parts.next().ok_or(())?),
                        status: FeatureSupported::Yes,
                    })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(qSupported(features))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum FeatureSupported {
    Yes,
    No,
    Maybe,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Feature<'a> {
    name: &'a str,
    val: Option<&'a str>,
    status: FeatureSupported,
}
