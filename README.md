# leecher-rs
Utility for downloading files from filehosters like Mediafire or Anonfiles automatically, in bulk and in the command line.

# Supports
 - Anonfiles
 - Mediafire
 - Pixeldrain
 - Direct Download

# Arguments
| Argument | Type   |                   Function                       |
|----------|--------|--------------------------------------------------|
| -q       | None   | Hides all console output except for progressbar. |
| Link**   | String | The link to the file on file hoster website.     |

** Accepts an unlimited number of arguments marked with this.
# Usage

Build it with cargo for your operating system (Only tested on windows!!!).
Supports pasting links as command line arguments and after running without arguments when seperated by a space.
You can use a direct download link either with a file name containing the whole url or with a specified file name using url[filename] (e.g. "https://ddl.com/FileToDownload.zip/download[File.zip]")

# Example:

![img](https://github.com/EKQRCalamity/leecher-rs/blob/main/preview.png)
