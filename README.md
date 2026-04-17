# Gerador de Senhas

Projeto em Rust com interface web para gerar senhas aleatorias, memoraveis e PINs seguros.

## Publicar no GitHub

Se o projeto ainda nao estiver em um repositorio remoto, rode no terminal:

```powershell
git init
git add .
git commit -m "Preparar deploy no Render"
git branch -M main
git remote add origin https://github.com/SEU-USUARIO/SEU-REPO.git
git push -u origin main
```

Se o repositorio ja existir, use so:

```powershell
git add .
git commit -m "Preparar deploy no Render"
git push
```

## Deploy gratis no Render

1. Envie este projeto para um repositorio no GitHub.
2. Crie uma conta no Render.
3. Escolha `New +` > `Blueprint` e selecione o repositorio.
4. O Render vai ler o arquivo `render.yaml` e criar o servico web.
5. Depois do deploy, o Render vai gerar uma URL publica parecida com:
   `https://gerador-senhas.onrender.com`
6. No painel do Render, confirme ou ajuste estas variaveis:
   - `SITE_NAME=Gerador de Senhas`
   - `SITE_URL=https://SEU-SERVICO.onrender.com`
   - `DOMAIN=SEU-SERVICO.onrender.com`
   - `GOOGLE_SITE_VERIFICATION=SEU_TOKEN_DO_GOOGLE` (opcional, para verificacao por meta tag)
7. Espere o deploy terminar e abra a URL publica.
8. Confirme que estas rotas respondem:
   - `/`
   - `/robots.txt`
   - `/sitemap.xml`

## Google Search Console

Como a hospedagem sera em um subdominio gratis do Render, adicione a propriedade como `Prefixo de URL`, nao como `Dominio`.

Use a URL publica completa do Render, por exemplo:

`https://gerador-senhas.onrender.com/`

Depois:

1. Se a verificacao por arquivo falhar, use o metodo `Tag HTML` no Google Search Console.
2. Copie apenas o valor do campo `content` da meta tag do Google.
3. No Render, adicione a variavel `GOOGLE_SITE_VERIFICATION` com esse valor e redeploy.
4. Verifique a propriedade novamente.
5. Envie o sitemap em:
   `https://gerador-senhas.onrender.com/sitemap.xml`
6. Use a inspecao de URL para solicitar indexacao da pagina inicial.
7. Aguarde o Google rastrear o site. Em hospedagem gratis isso pode levar algum tempo.

## Checklist final

1. O projeto esta no GitHub.
2. O Render gerou uma URL `*.onrender.com` publica.
3. `robots.txt` abre sem erro.
4. `sitemap.xml` abre sem erro.
5. A propriedade foi adicionada no Search Console como `Prefixo de URL`.
6. O sitemap foi enviado.

## Desenvolvimento local

```powershell
cargo run
```

O servidor usa a variavel `PORT` quando ela existir. Sem isso, roda em `3000`.
