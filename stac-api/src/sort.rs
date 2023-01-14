use std::{convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};

/// Fields by which to sort results.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Sortby {
    /// The field to sort by.
    pub field: String,

    /// The direction to sort by.
    pub direction: Direction,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Direction {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

impl Sortby {
    /// Creates a new ascending sortby for the field.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::Sortby;
    /// let sortby = Sortby::asc("id");
    /// ```
    pub fn asc(field: impl ToString) -> Sortby {
        Sortby {
            field: field.to_string(),
            direction: Direction::Ascending,
        }
    }

    /// Creates a new descending sortby for the field.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::Sortby;
    /// let sortby = Sortby::desc("id");
    /// ```
    pub fn desc(field: impl ToString) -> Sortby {
        Sortby {
            field: field.to_string(),
            direction: Direction::Descending,
        }
    }

    /// Creates a vector of [Sortbys](Sortby) from a comma-delimited list.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::Sortby;
    /// let sortbys = Sortby::from_query_param("+id,-datetime");
    /// ```
    pub fn from_query_param(s: &str) -> Vec<Sortby> {
        s.split(',')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.parse().unwrap()) // infallible
                }
            })
            .collect()
    }
}

impl FromStr for Sortby {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('+') {
            Ok(Sortby::asc(&s[1..]))
        } else if s.starts_with('-') {
            Ok(Sortby::desc(&s[1..]))
        } else {
            Ok(Sortby::asc(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sortby;
    use serde_json::json;

    #[test]
    fn optional_plus() {
        assert_eq!(
            "properties.created".parse::<Sortby>().unwrap(),
            "+properties.created".parse().unwrap()
        );
    }

    #[test]
    fn descending() {
        assert_eq!(Sortby::desc("id"), "-id".parse().unwrap());
    }

    #[test]
    fn ordering() {
        assert_eq!(
            vec![
                Sortby::asc("properties.created"),
                Sortby::desc("properties.eo:cloud_cover"),
                Sortby::desc("id"),
                Sortby::asc("collection")
            ],
            Sortby::from_query_param(
                "+properties.created,-properties.eo:cloud_cover,-id,collection"
            )
        )
    }

    #[test]
    fn names() {
        assert_eq!(
            json!({"field": "foo", "direction": "asc"}),
            serde_json::to_value(Sortby::asc("foo")).unwrap()
        );
        assert_eq!(
            json!({"field": "foo", "direction": "desc"}),
            serde_json::to_value(Sortby::desc("foo")).unwrap()
        );
    }
}
