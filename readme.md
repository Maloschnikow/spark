### Development
might wanna use the [Nix](https://nixos.org/download/) packet manager with this one

You can also additionally use [nix-direnv](https://github.com/nix-community/nix-direnv). In that case you probably also want the `mkhl.direnv` extension for vscode.\
If you're using Nix HomeManager include this in your configuration and you should be good to go.
```nix
programs.direnv = {
    enable = true;
    nix-direnv.enable = true;
    enableZshIntegration = true;
    #enableBashIntegration = true;
};
```
In case you don't wanna make the additional effort and only use Nix, you can simply run
`nix develop` to end up in a development shell with the right environment.\
Oh and if you're using vscode, you might also wanna use the recommended extensions.

#### Backend
First `cd backend/`.

Audit packages with `cargo-audit audit` and `cargo-audit audit fix`.\
Run with `cargo run`.\
Find API documentation under `localhost:3000/doc`.

#### Database
The password is `password`.

Start/Initialize Database with `db-start`.\
Stop Database with `db-stop`.\
Reset and restart Database with `db-reset`.

Migrate up every migration with `db-upgrade`.\
Migrate down **one** migration with `db-downgrade`.

Use `database/dev_init.sql` to fill the database with example data.