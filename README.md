# Mail client

A mail client that runs in a window of its own or, from a console, as a
terminal program. It reads and sends mail over IMAP and SMTP, with a password
or a browser sign-in.

This file is the plain statement of what the program keeps, who it talks to,
and how to remove it. The same facts are on the Data page of the Options screen
(`F10`).

## What it keeps

Everything it keeps is in one folder, the data folder: a `data` folder beside
the program, or the folder named with `--data-dir <folder>`. Nothing is written
anywhere else.

| Where in the data folder | What it holds |
| --- | --- |
| `accounts/<id>/account.toml` | the account: server names, user name, and the password if one is used (plain text) |
| `accounts/<id>/oauth/<id>.token` | the sign-in tokens of a browser sign-in (plain text) |
| `accounts/<id>/sent_copies/` | copies of sent messages not yet saved in the Sent folder; each goes when saved |
| `accounts/<id>/account.toml.bak` | one backup of the account file, without its secrets |
| `config.toml` | your settings |
| `state.toml` | the sizes of the panes and the window |
| `addresses.txt` | the names and addresses of the people your mail came from, went to or was copied to, and those you sent to, so they can complete as you type a recipient (plain text, one per line, at most 2000, only you can read it; delete the file to forget them) |
| `html/` | the safe copies of HTML messages you opened in your browser with `b` (`mail-000001.html` and so on; scripts, frames, forms and everything that loads from the network are left out; only you can read them; at most 20 are kept and the ones left from an earlier run are removed when the program starts; not made until you open one) |
| `oauth_clients.toml` | the client ids (and any client secret) you entered for browser sign-in, kept once per provider |
| `themes/` | themes you added |
| `lock` | lets one copy of the program use the folder at a time |

Mail itself is not stored on disk: messages are read from the server when
shown and kept in memory only, except the safe copy of an HTML message that you
ask to open in your browser (the `html/` folder above). The program writes no log.

On Linux and macOS the folders and files are limited to your user. On Windows
they are not yet protected beyond your account's own folder permissions, and the
passwords and tokens are plain text on every system: do not copy the data folder
to a place others can read.

## Who it contacts

- The mail servers you set up, over TLS.
- For a browser sign-in, the provider's sign-in service, through your own
  browser and a short-lived listener on this machine.
- Links you open yourself, in your browser, and the safe copy of a message you
  open in your browser (a file on this machine).

It sends nothing to its author: no usage figures, no crash reports, no update
checks, and it loads no remote images or content from messages.

## Removing it

- **One account:** remove it in Options. Its folder (account file, tokens, saved
  copies, backup) is deleted. Your settings, the saved client ids and the address list stay.
- **Everything:** close the program and delete the data folder. The program
  itself is a single file; delete it too.

## Licence

BS Mail is not open source. It is free for your own personal, non-commercial
use: you may build it and run it on your own computers and read its source.
You may not copy or share it, change it, sell it, offer it as a service, or use
it commercially, without written permission. See `LICENSE`. The fonts and
libraries it uses keep their own licenses, which are kept in the `licenses`
folder beside the program.
