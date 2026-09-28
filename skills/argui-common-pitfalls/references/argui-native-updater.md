# Native update integration

Use one application-owned `argui-updater::Updater` session for check,
download, cancellation and explicit installation. Expose typed snapshots to
every UI scene. Keep network work on a service worker, outside window startup;
retain a separate readable snapshot and cancellation token so opening About
and cancelling a download never wait for the download mutex.

Choose the trust boundary deliberately. ARGUI's Minisign HTTP backend needs
a persistent public key compiled into the application and the matching private
key kept by release CI. Generating a fresh key pair on each CI run cannot make
older applications verify the next release. A custom GitHub HTTPS backend can
instead use release metadata as its trust source; describe that explicitly,
because a SHA-256 supplied by the same publisher is an integrity check, not
a signature.

Before downloading, freeze the chosen platform URL, format, size and SHA-256.
Limit feed and package sizes, validate HTTPS hosts and redirect destinations,
reject malformed or missing target records, and enforce the frozen size/hash
while streaming into a private temporary file. Recheck cancellation after
reads and before returning a verified package. Publish this verified package
to the native installer only after the complete stream succeeds.

If the UI binary depends on JS bundles, fonts, a capture engine or a private
runtime, update the whole application package. Replacing the UI executable
alone leaves those versions inconsistent. Stage every native architecture in
CI before packaging. On macOS, relocate the UI companion's SDK dylib paths
into the application's adjacent private runtime before archiving the `.app`.
Retain required font licenses alongside bundled assets.

Source checkouts may check and download releases, but the installer must
validate the installed package type and application path. Installation should
reject active capture and require an explicit user action. Keep checking
updates separate from downloading, installing or restarting.
