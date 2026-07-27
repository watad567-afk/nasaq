# Publish guide — Nasaq public launch

## GitHub (done)

Repository: **https://github.com/watad567-afk/nasaq**

### Enable CI + Pages workflows

The first push omitted `.github/workflows/` because the GitHub CLI token needs the `workflow` scope.

1. Open https://github.com/login/device and enter code from:
   ```powershell
   gh auth refresh -h github.com -s workflow
   ```
2. Push workflows:
   ```powershell
   git add .github/workflows
   git commit -m "ci: add GitHub Actions workflows"
   git push
   ```
3. In GitHub → **Settings → Pages → Build and deployment**: set source to **GitHub Actions**.

## npm (requires login)

```powershell
npm login
cd crates/nasaq_runtime/npm
copy ..\js\*.js .
npm publish --access public

cd ../../../npm/nasaq-lang
npm publish --access public
```

Or set `NODE_AUTH_TOKEN` in GitHub secrets and use the release workflows.

## Local website

```powershell
cargo build --release -p nasaq_cli --target-dir target3
.\target3\release\nasaq.exe website --port 8080
```

Open http://localhost:8080

## Install for developers

```powershell
cargo install --path crates/nasaq_cli
nasaq new myapp --template web
```
