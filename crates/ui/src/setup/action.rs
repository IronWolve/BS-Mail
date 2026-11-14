//! What the form asks the program to do that is not a file or a server: the
//! program owns the web browser, the form only asks. (A link to copy goes out
//! through the same channel as text copied in the composer.)

/// One request from the sign-in page, taken once by the program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupAction {
    /// Open this link in the user's web browser.
    OpenLink(String),
    /// Stop waiting for the browser and go back to the form.
    CancelSignIn,
    /// The form was closed without saving: stop any attempt still going.
    Abandon,
}
