// ext
extern crate hyper;
extern crate url;
// traits
use std::clone::Clone;
use std::fmt;
use std::string::ToString;

pub struct HumanURI {
    // TODO: store path components and all that separately
    url: hyper::Url
}

impl HumanURI {
    fn wrap(url: hyper::Url) -> Self {
        HumanURI{
            url: url
        }
    }

    pub fn parse(raw_uri: &str) -> Self {
        let url = if raw_uri.starts_with("http://") || raw_uri.starts_with("https://") {
            hyper::Url::parse(raw_uri)
        } else {
            hyper::Url::parse(&format!("http://{}", raw_uri))
        }.unwrap();
        Self::wrap(url)
    }

    // XXX: with_* and add_* methods are very similiar %(
    pub fn with_query_params<'a, K, V, I>(&self, params: I) -> Self
        where I: Iterator<Item=&'a (K, V)>,
              K: 'a + AsRef<str>,
              V: 'a + ToString
    {
        let mut new_url = self.url.clone();
        // Save params to Vec, to keep created Strings for V there
        let sparams = params.map(|pair| {
            let ref k = pair.0;
            let ref v = pair.1;
            (k, v.to_string())
        }).collect::<Vec<_>>();
        // Pass references to passed K and created String for V to
        // setter
        new_url.set_query_from_pairs(sparams.iter()
                                            .map(|&(ref k, ref v)|
                                                 (k.as_ref(), v.as_str())));
        Self::wrap(new_url)
    }

    pub fn add_query_params<'a, K, V, I>(&self, params: I) -> Self
        where I: Iterator<Item=&'a (K, V)>,
              K: 'a + AsRef<str>,
              V: 'a + ToString
    {
        let mut new_url = self.url.clone();
        // Save params to Vec, to keep created Strings for V there
        let sparams = params.map(|pair| {
            let ref k = pair.0;
            let ref v = pair.1;
            (k, v.to_string())
        }).collect::<Vec<_>>();
        // Extract current query
        let current_query = match self.url.query_pairs() {
            Some(cq) => cq,
            None => vec!()
        };
        // Create Iter<&str, &str> from current query params
        let current_i = current_query.iter().map(|&(ref k, ref v)|
                                                 (k.as_str(), v.as_str()));
        // Create Iter from passed K and Strings created from V
        let sparams_i = sparams.iter().map(|&(ref k, ref v)|
                                           (k.as_ref(), v.as_str()));
        // Chain current query with receieved params
        let new_query = current_i.chain(sparams_i);
        new_url.set_query_from_pairs(new_query);
        Self::wrap(new_url)
    }

    /// Returns new uri, by appending path components
    pub fn with_path_components<E, I>(&self, paths: I) -> Self
        where I: Iterator<Item=E>,
              E: AsRef<str>
    {
        let mut new_url = self.url.clone();
        // Protect from "borrow of `new_url` occurs here"
        {
            // we will unwrap, because we want to use this for http urls only
            let mut path_components = new_url.path_mut().unwrap();
            path_components.clear();
            path_components.extend(paths.map(|s| s.as_ref().to_string()));
        }
        Self::wrap(new_url)
    }

    pub fn add_path_components<E, I>(&self, paths: I) -> Self
        where I: Iterator<Item=E>,
              E: AsRef<str>
    {
        let mut new_url = self.url.clone();
        {
            // we will unwrap, because we want to use this for http urls only
            let mut path_components = new_url.path_mut().unwrap();
            path_components.extend(paths.map(|s| s.as_ref().to_string()));
        }
        Self::wrap(new_url)
    }

    pub fn add_path<T:AsRef<str>>(&self, path: T) -> Self {
        self.add_path_components(path.as_ref()
                                 .trim_left_matches('/')
                                 .split("/"))
    }

    pub fn with_path<T:AsRef<str>>(&self, path: T) -> Self {
        self.with_path_components(path.as_ref()
                                  .trim_left_matches('/')
                                  .split("/"))
    }

}

impl hyper::client::IntoUrl for HumanURI {
    fn into_url(self) -> Result<hyper::Url, url::ParseError> {
        Ok(self.url)
    }
}

impl fmt::Display for HumanURI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.url.fmt(f)
    }
}
