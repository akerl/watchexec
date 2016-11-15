use std::io;
use std::path::Path;

use globset;
use globset::{Glob, GlobSet, GlobSetBuilder};

use ignore::Ignore;

pub struct NotificationFilter {
    filters: GlobSet,
    filter_count: usize,
    ignores: GlobSet,
    ignore: Option<Ignore>,
}

#[derive(Debug)]
pub enum Error {
    Glob(globset::Error),
    Io(io::Error),
}

impl NotificationFilter {
    pub fn new(filters: Vec<String>,
               ignores: Vec<String>,
               ignore: Option<Ignore>)
               -> Result<NotificationFilter, Error> {
        let mut filter_set_builder = GlobSetBuilder::new();
        for f in &filters {
            filter_set_builder.add(try!(Glob::new(f)));

            debug!("Adding filter: \"{}\"", f);
        }

        let mut ignore_set_builder = GlobSetBuilder::new();
        for i in &ignores {
            ignore_set_builder.add(try!(Glob::new(i)));

            debug!("Adding ignore: \"{}\"", i);
        }

        let filter_set = try!(filter_set_builder.build());
        let ignore_set = try!(ignore_set_builder.build());

        Ok(NotificationFilter {
            filters: filter_set,
            filter_count: filters.len(),
            ignores: ignore_set,
            ignore: ignore,
        })
    }

    pub fn is_excluded(&self, path: &Path) -> bool {
        if self.ignores.is_match(path) {
            debug!("Ignoring {:?}: matched ignore filter", path);
            return true;
        }

        if self.filters.is_match(path) {
            return false;
        }

        if let Some(ref ign) = self.ignore {
            if ign.is_excluded(path) {
                debug!("Ignoring {:?}: matched gitignore file", path);
                return true;
            }
        }

        if self.filter_count > 0 {
            debug!("Ignoring {:?}: did not match any given filters", path);
        }

        self.filter_count > 0
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<globset::Error> for Error {
    fn from(err: globset::Error) -> Error {
        Error::Glob(err)
    }
}

#[cfg(test)]
mod tests {
    use super::NotificationFilter;
    use std::path::Path;

    #[test]
    fn test_allows_everything_by_default() {
        let filter = NotificationFilter::new(vec![], vec![], None).unwrap();

        assert!(!filter.is_excluded(&Path::new("foo")));
    }

    #[test]
    fn test_multiple_filters() {
        let filters = vec![String::from("*.rs"), String::from("*.toml")];
        let filter = NotificationFilter::new(filters, vec![], None).unwrap();

        assert!(!filter.is_excluded(&Path::new("hello.rs")));
        assert!(!filter.is_excluded(&Path::new("Cargo.toml")));
        assert!(filter.is_excluded(&Path::new("README.md")));
    }

    #[test]
    fn test_multiple_ignores() {
        let ignores = vec![String::from("*.rs"), String::from("*.toml")];
        let filter = NotificationFilter::new(vec![], ignores, None).unwrap();

        assert!(filter.is_excluded(&Path::new("hello.rs")));
        assert!(filter.is_excluded(&Path::new("Cargo.toml")));
        assert!(!filter.is_excluded(&Path::new("README.md")));
    }

    #[test]
    fn test_ignores_take_precedence() {
        let ignores = vec![String::from("*.rs"), String::from("*.toml")];
        let filter = NotificationFilter::new(ignores.clone(), ignores, None).unwrap();

        assert!(filter.is_excluded(&Path::new("hello.rs")));
        assert!(filter.is_excluded(&Path::new("Cargo.toml")));
        assert!(filter.is_excluded(&Path::new("README.md")));
    }
}
