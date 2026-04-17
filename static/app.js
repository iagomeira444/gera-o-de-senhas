// ============================================================
//  ELEMENTOS
// ============================================================
const $ = (id) => document.getElementById(id);

const senhaDisplay = $("senhaDisplay");
const forcaProgresso = $("forcaProgresso");
const forcaRotulo = $("forcaRotulo");
const btnGerarPrincipal = $("btnGerarPrincipal");
const btnGerar = $("btnGerar");
const btnCopiar = $("btnCopiar");
const toast = $("toast");

const painelAleatorio = $("painelAleatorio");
const painelMemoravel = $("painelMemoravel");
const painelPin = $("painelPin");

let tipoAtivo = "aleatorio";

function createRefreshIcon() {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("width", "20");
    svg.setAttribute("height", "20");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "currentColor");
    svg.setAttribute("stroke-width", "2");

    const path1 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    path1.setAttribute("d", "M23 4v6h-6M1 20v-6h6");

    const path2 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    path2.setAttribute("d", "M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15");

    svg.appendChild(path1);
    svg.appendChild(path2);
    return svg;
}

function renderGenerateButton(isLoading) {
    btnGerarPrincipal.replaceChildren();

    if (isLoading) {
        const spinner = document.createElement("div");
        spinner.className = "spinner";

        btnGerarPrincipal.appendChild(spinner);
        btnGerarPrincipal.appendChild(document.createTextNode(" Gerando..."));
        return;
    }

    btnGerarPrincipal.appendChild(createRefreshIcon());
    btnGerarPrincipal.appendChild(document.createTextNode(" Gerar Senha Segura"));
}

// ============================================================
//  API - COMUNICAÇÃO COM BACKEND RUST
// ============================================================

async function chamarAPI(endpoint, dados) {
    const controller = new AbortController();
    const timeoutId = window.setTimeout(() => controller.abort(), 8000);

    try {
        const resp = await fetch(endpoint, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(dados),
            signal: controller.signal,
        });

        const contentType = resp.headers.get("content-type") || "";
        const isJson = contentType.includes("application/json");
        const payload = isJson ? await resp.json() : null;

        if (!resp.ok) {
            const message = payload && typeof payload.erro === "string"
                ? payload.erro
                : `Erro ${resp.status}`;
            const error = new Error(message);
            error.status = resp.status;
            throw error;
        }

        return payload;
    } catch (error) {
        if (error.name === "AbortError") {
            throw new Error("Tempo limite excedido. Tente novamente.");
        }

        throw error;
    } finally {
        window.clearTimeout(timeoutId);
    }
}

// ============================================================
//  GERAR SENHA (via API)
// ============================================================

let gerando = false;

async function gerarSenha() {
    if (gerando) return;
    gerando = true;
    btnGerarPrincipal.disabled = true;
    btnGerar.disabled = true;
    renderGenerateButton(true);

    try {
        let dados = { tipo: tipoAtivo };

        if (tipoAtivo === "aleatorio") {
            dados.tamanho = parseInt($("tamanho").value);
            dados.maiusculas = $("chkMaiusculas").checked;
            dados.numeros = $("chkNumeros").checked;
            dados.simbolos = $("chkSimbolos").checked;
            dados.excluir_ambiguos = $("chkAmbiguos").checked;
        } else if (tipoAtivo === "memoravel") {
            dados.num_palavras = parseInt($("numPalavras").value);
            dados.separador = $("separador").value;
            dados.capitalizar = $("chkCapitalizar").checked;
            dados.incluir_numero = $("chkIncluirNum").checked;
        } else if (tipoAtivo === "pin") {
            dados.tamanho_pin = parseInt($("tamanhoPin").value);
        }

        const resultado = await chamarAPI("/api/gerar", dados);

        // Exibir senha
        senhaDisplay.textContent = resultado.senha;

        // Exibir força
        const f = resultado.forca;
        forcaProgresso.style.width = f.porcentagem + "%";
        forcaProgresso.style.background = f.cor;
        forcaRotulo.textContent = f.rotulo;
        forcaRotulo.style.color = f.cor;
    } catch (err) {
        senhaDisplay.textContent = "Erro ao gerar senha";
        if (err.status === 429) {
            mostrarToast("Muitas tentativas. Aguarde um instante.");
        } else {
            mostrarToast(err.message || "Falha ao gerar senha.");
        }

        console.error("Falha ao gerar senha", {
            status: err.status || null,
            message: err.message,
        });
    } finally {
        gerando = false;
        btnGerarPrincipal.disabled = false;
        btnGerar.disabled = false;
        renderGenerateButton(false);
    }
}

// ============================================================
//  COPIAR
// ============================================================

async function copiarSenha() {
    const texto = senhaDisplay.textContent;
    if (!texto || texto === "Clique em gerar...") return;

    try {
        await navigator.clipboard.writeText(texto);
        mostrarToast("Senha copiada!");
    } catch {
        // Fallback
        const ta = document.createElement("textarea");
        ta.value = texto;
        ta.style.position = "fixed";
        ta.style.opacity = "0";
        document.body.appendChild(ta);
        ta.select();
        document.execCommand("copy");
        document.body.removeChild(ta);
        mostrarToast("Senha copiada!");
    }
}

function mostrarToast(msg) {
    toast.textContent = msg;
    toast.classList.add("visivel");
    setTimeout(() => toast.classList.remove("visivel"), 2000);
}

// ============================================================
//  TROCAR ABAS
// ============================================================

function trocarAba(tipo) {
    tipoAtivo = tipo;

    document.querySelectorAll(".tab").forEach((t) => {
        t.classList.toggle("active", t.dataset.tipo === tipo);
    });

    painelAleatorio.classList.toggle("oculto", tipo !== "aleatorio");
    painelMemoravel.classList.toggle("oculto", tipo !== "memoravel");
    painelPin.classList.toggle("oculto", tipo !== "pin");

    gerarSenha();
}

// ============================================================
//  EVENTOS
// ============================================================

// Botões
btnGerarPrincipal.addEventListener("click", gerarSenha);
btnGerar.addEventListener("click", gerarSenha);
btnCopiar.addEventListener("click", copiarSenha);

// Tabs
document.querySelectorAll(".tab").forEach((tab) => {
    tab.addEventListener("click", () => trocarAba(tab.dataset.tipo));
});

// Sliders
$("tamanho").addEventListener("input", (e) => {
    $("tamanhoValor").textContent = e.target.value;
    gerarSenha();
});

$("numPalavras").addEventListener("input", (e) => {
    $("palavrasValor").textContent = e.target.value;
    gerarSenha();
});

$("tamanhoPin").addEventListener("input", (e) => {
    $("pinValor").textContent = e.target.value;
    gerarSenha();
});

// Checkboxes e Select
["chkMaiusculas", "chkNumeros", "chkSimbolos", "chkAmbiguos",
    "chkCapitalizar", "chkIncluirNum"].forEach((id) => {
        $(id).addEventListener("change", gerarSenha);
    });

$("separador").addEventListener("change", gerarSenha);

// ============================================================
//  INICIALIZAR
// ============================================================

renderGenerateButton(false);
gerarSenha();
