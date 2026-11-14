//! The rows of the account form and the choices some of them make.

/// How a connection is protected, as the account file spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Security {
    Implicit,
    Starttls,
    /// No encryption: for a server on this same machine only.
    None,
}

impl Security {
    pub fn label(self) -> &'static str {
        match self {
            Self::Implicit => "encrypted from the start (TLS)",
            Self::Starttls => "encrypted after hello (STARTTLS)",
            Self::None => "not encrypted (this machine only)",
        }
    }

    /// The choice an account file's word names, `None` for any other word.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "implicit" => Some(Self::Implicit),
            "starttls" => Some(Self::Starttls),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn toml_name(self) -> &'static str {
        match self {
            Self::Implicit => "implicit",
            Self::Starttls => "starttls",
            Self::None => "none",
        }
    }

    /// The next choice, wrapping.
    pub fn next(self) -> Self {
        match self {
            Self::Implicit => Self::Starttls,
            Self::Starttls => Self::None,
            Self::None => Self::Implicit,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Implicit => Self::None,
            Self::Starttls => Self::Implicit,
            Self::None => Self::Starttls,
        }
    }
}

/// How the user proves who they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignIn {
    /// A password, or an app password the provider made.
    Password,
    /// The provider's own page in a web browser; no password is typed here.
    Browser,
}

impl SignIn {
    pub fn label(self) -> &'static str {
        match self {
            Self::Password => "a password or app password",
            Self::Browser => "your web browser (OAuth2)",
        }
    }

    /// The other choice.
    pub fn other(self) -> Self {
        match self {
            Self::Password => Self::Browser,
            Self::Browser => Self::Password,
        }
    }
}

/// The rows of the form, in the order Tab visits them. Which of them show
/// depends on how the user signs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupField {
    Email,
    SignIn,
    Password,
    ClientId,
    ClientSecret,
    Name,
    ImapHost,
    ImapPort,
    ImapSecurity,
    SmtpHost,
    SmtpPort,
    SmtpSecurity,
    Username,
    AllowChanges,
    Save,
}

impl SetupField {
    pub const ALL: [SetupField; 15] = [
        Self::Email,
        Self::SignIn,
        Self::Password,
        Self::ClientId,
        Self::ClientSecret,
        Self::Name,
        Self::ImapHost,
        Self::ImapPort,
        Self::ImapSecurity,
        Self::SmtpHost,
        Self::SmtpPort,
        Self::SmtpSecurity,
        Self::Username,
        Self::AllowChanges,
        Self::Save,
    ];

    /// The next row in the list, shown or not.
    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|field| *field == self).unwrap_or(0)
    }

    /// Whether the row shows for this way of signing in.
    pub fn is_shown(self, signin: SignIn) -> bool {
        match self {
            Self::Password => signin == SignIn::Password,
            Self::ClientId | Self::ClientSecret => signin == SignIn::Browser,
            _ => true,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Email => "Email address",
            Self::SignIn => "Sign in with",
            Self::Password => "Password",
            Self::ClientId => "Client id",
            Self::ClientSecret => "Client secret",
            Self::Name => "Your name",
            Self::ImapHost => "Incoming server",
            Self::ImapPort | Self::SmtpPort => "port",
            Self::ImapSecurity | Self::SmtpSecurity => "  security",
            Self::SmtpHost => "Outgoing server",
            Self::Username => "User name",
            Self::AllowChanges => "Let me change mail",
            Self::Save => "",
        }
    }

    /// What an empty text row says it wants, drawn dimly until it has focus.
    pub fn placeholder(self) -> &'static str {
        match self {
            Self::ClientId => "needed once: from your provider's developer console",
            Self::ClientSecret => "only if the console gave one",
            _ => "",
        }
    }

    /// A field that is a choice and not text.
    pub fn is_choice(self) -> bool {
        matches!(
            self,
            Self::SignIn | Self::ImapSecurity | Self::SmtpSecurity | Self::AllowChanges | Self::Save
        )
    }

    /// A text row drawn as dots, never copied out.
    pub fn is_secret(self) -> bool {
        matches!(self, Self::Password | Self::ClientSecret)
    }

    pub(super) fn is_server(self) -> bool {
        matches!(
            self,
            Self::ImapHost | Self::ImapPort | Self::ImapSecurity | Self::SmtpHost | Self::SmtpPort | Self::SmtpSecurity
        )
    }
}
