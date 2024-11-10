# `conflink` - easy config managing

`conflink` is a utility tool designed for easy file and directory symlinking. In
particular, this makes it easy to handle your config files.

## Usage

`conflink` uses a config file (written in TOML) that defines how your symlinks
should look like. There's no default configuration that is used, because
everyone's structure is slightly different, and it is preffered to be explicit
about it. Otherwise, it might be possible to overwrite configurations with some
nonsense.

To generate a _default_ template configuration, use `conflink --gen-config`

## Config file

The configuration file specifies what to symlink and where.

### Config file examples:

#### Link all files

This configuration will create symlinks in `working-dir` for all files and
directories contained in `link-from-dir`.

```toml
[conflink]
working-dir = "path/to/working/directory"
link-from-dir = "path/to/dir/containing/files-and-dirs"
link-all = true
```

#### Per-link config

You might have specific configurations for different devices. For example, I use
both macos and Linux, so some configurations are specific to target os. In this
case, I use something like `app_name/hostname/config-file` and link the specific
config file (or directory) depending on the os. To achieve this, you can do
something like the following:

```toml
[conflink]
# link all dirs and files
working-dir = "path/to/working/directory"
link-from-dir = "path/to/dir/containing/files-and-dirs"
link-all = true

# specific link for `app_name` with no conditionals
[conflink.app_name]
link-path = "$HOME/.config/app_name"
link-to = "$HOME/dotfiles/app_name"

# specific link for `some_app` when $hostname == 'some-hostname'
[conflink.'eq($hostname, "some-hostname")'.some_app]
link-path = "$HOME/.config/app_name"
link-to = "$HOME/dotfiles/app_name/host_specific"
```
