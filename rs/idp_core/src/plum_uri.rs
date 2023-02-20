use idp_proto::PlumHeadSeal;

#[derive(
    Clone,
    Debug,
    derive_more::Deref,
    serde::Deserialize,
    derive_more::From,
    derive_more::Into,
    serde::Serialize,
)]
pub struct PlumURILocal(PlumHeadSeal);

impl std::fmt::Display for PlumURILocal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "idp:///{}", self.0)
    }
}

impl PlumURILocal {
    pub fn get_plum_head_seal(&self) -> &PlumHeadSeal {
        &self.0
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PlumURIRemote {
    pub hostname: String,
    pub port_o: Option<u16>,
    pub plum_head_seal: PlumHeadSeal,
    // TODO: Figure out appropriate way to support server- and client-side queries.
    // The easiest would just be to have client-side-only fragment queries, like
    // ordinary HTTP resources, because it would just involve a client-side query
    // on a Plum that was retrieved.
}

impl std::fmt::Display for PlumURIRemote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if let Some(port) = self.port_o {
            write!(
                f,
                "idp://{}:{}/{}",
                self.hostname, port, self.plum_head_seal
            )
        } else {
            write!(f, "idp://{}/{}", self.hostname, self.plum_head_seal)
        }
    }
}

impl PlumURIRemote {
    pub fn remote_server_url(&self) -> String {
        let scheme = if self.hostname_is_local() {
            "http://"
        } else {
            "https://"
        };
        if let Some(port) = self.port_o {
            format!("{}{}:{}", scheme, self.hostname, port)
        } else {
            format!("{}{}", scheme, self.hostname)
        }
    }
    pub fn get_plum_head_seal(&self) -> &PlumHeadSeal {
        &self.plum_head_seal
    }

    fn hostname_is_local(&self) -> bool {
        match self.hostname.as_str() {
            "localhost" | "127.0.0.1" | "0.0.0.0" => true,
            _ => false,
        }
    }
}

#[derive(
    Clone, Debug, serde::Deserialize, derive_more::Display, derive_more::From, serde::Serialize,
)]
pub enum PlumURI {
    Local(PlumURILocal),
    Remote(PlumURIRemote),
}

// TODO: PlumURI parsing from str

impl From<PlumHeadSeal> for PlumURI {
    fn from(plum_head_seal: PlumHeadSeal) -> Self {
        PlumURI::from(PlumURILocal::from(plum_head_seal))
    }
}

impl PlumURI {
    pub fn get_plum_head_seal(&self) -> &PlumHeadSeal {
        match self {
            PlumURI::Local(x) => x.get_plum_head_seal(),
            PlumURI::Remote(x) => x.get_plum_head_seal(),
        }
    }
}
