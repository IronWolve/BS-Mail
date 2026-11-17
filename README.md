# BS Mail

*Mail for Power Users*

A fast, keyboard-first mail client for any IMAP and SMTP account. It opens in a
window of its own, and on Linux it can also run inside a terminal. Everything
works with the mouse too.

![BS Mail: folders, message list and the open message](screenshots/main.png)

## Download

Get the latest release from the [Releases page](../../releases).

| System | File |
| --- | --- |
| Linux, 64-bit (Ubuntu 22.04 or newer, and most other current distributions) | `BSMail_linux-x64_v0.9.1.AppImage` |
| Windows 10 or 11, 64-bit | `BSMail_windows-x64_v0.9.1.zip` |

Each file has a `.sha256` file beside it so you can check the download.

- **Linux:** make the file executable (`chmod +x BSMail_linux-x64_v0.9.1.AppImage`) and run it.
  If it does not start, your system may need FUSE: install `libfuse2` (`libfuse2t64` on Ubuntu 24.04).
  Started from a terminal it runs in that terminal; add `--window` to get the window.
- **Windows:** unzip, then run `bs-mail.exe`. The program is not signed, so Windows may say
  "Windows protected your PC": choose *More info*, then *Run anyway*.

## First steps

1. Start BS Mail. It opens with no account.
2. Press `F10` (or click the Options button), choose **Accounts**, then **Add account**.
3. Type your email address and pick your provider from the list: Gmail, Outlook, iCloud,
   Yahoo, Fastmail, GMX, Zoho, Proton Mail Bridge, 1&1 / IONOS and many more fill in the
   servers for you. Enter your password. If your provider asks for an app password, make
   one in your account settings and use that.
4. Your mail appears. Press `?` for every key.

Signing in through your web browser (OAuth2) is there too, for providers that offer it;
it needs a client ID that you make once with the provider.

## What it does

- **Several accounts** at once, each with its own folders, switched with a click or the keyboard.
- **Read and write:** reply, reply to everyone, forward, with attachments. Pick which account
  sends, keep a signature, and complete addresses from the people you wrote to.
- **Attachments:** save them or open them. Programs and scripts are saved but never opened.
- **HTML mail** opens as a safe copy in your browser: scripts, frames, forms and everything that
  would load from the internet are left out, and it tells you how many items were removed.
- **Find and arrange:** search, sort by any column, group into threads, tag, star and mark alerts.
  The Starred and Alerts views search the whole account.
- **Many at once:** mark several messages (`Space`, Ctrl or Shift click) and delete, move, tag,
  mark read, star or archive them together. Delete always asks first.
- **Folders:** make, rename and delete them, mark all read, empty the Trash or Spam.
- **Mouse and keyboard:** right-click menus, drag messages onto a folder, click links and addresses,
  select and copy text, find inside a message, view the headers or the whole source.
- **Make it yours:** layout and panels in Options, any key rebound in `config.toml`, your own themes.

![Writing a reply](screenshots/reply.png)

## Keys to start with

| Key | Does |
| --- | --- |
| `j` `k` or arrows | move |
| `Enter`, `Esc` | go in, come back out |
| `c` | write a message |
| `r`, `R`, `f` | reply, reply to everyone, forward |
| `d` | delete (asks first) |
| `Space` | mark a message; then act on all marked |
| `/` | search |
| `o`, `O`, `T` | sort, reverse the sort, threads on or off |
| `t` | tag |
| `.` or right-click | the menu |
| `F10` | options |
| `?` | all keys |

![The menu of a message](screenshots/menu.png)

## Your data and privacy

- Everything BS Mail keeps is in one folder and nowhere else: the `data` folder next to the
  program on Windows, and `~/.bs-mail` for the Linux AppImage. Mail itself is not stored on
  your computer; it is read from the server when you open it.
- It talks only to the mail servers you set up (over TLS), to your provider's sign-in page if
  you use the browser sign-in, and to your browser for links you click. It sends nothing to
  its author: no usage figures, no crash reports, no update checks, and it never loads remote
  images from messages.
- Passwords are kept as plain text in the account file inside that folder, so do not copy the
  folder to a place other people can read.
- To remove everything, delete the program and that folder.

## Licence

BS Mail is not open source. It is free for your own personal, non-commercial use. You may not
copy or share it, change it, sell it, or use it commercially without written permission. See
[LICENSE](LICENSE). The fonts and libraries it uses keep their own licenses, which come with
each download in the file `NOTICE`.
