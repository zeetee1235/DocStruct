const inputListEl = document.getElementById("inputList");
const outputDirEl = document.getElementById("outputDir");
const dpiEl = document.getElementById("dpi");
const convertBtn = document.getElementById("convertBtn");
const pickInputBtn = document.getElementById("pickInput");
const pickOutputBtn = document.getElementById("pickOutput");
const copyBtn = document.getElementById("copyBtn");
const statusEl = document.getElementById("status");
const resultTextEl = document.getElementById("resultText");

const invoke = window.__TAURI__?.core?.invoke;
let progressTimerId = null;
let selectedInputPaths = [];

function setStatus(message) {
  statusEl.textContent = message;
}

function setBusy(busy) {
  convertBtn.disabled = busy;
  pickInputBtn.disabled = busy;
  pickOutputBtn.disabled = busy;
  copyBtn.disabled = busy;
  convertBtn.textContent = busy ? "Converting..." : "Convert";
}

function stopProgressTimer() {
  if (progressTimerId !== null) {
    clearInterval(progressTimerId);
    progressTimerId = null;
  }
}

function startProgressTimer(startedAtMs) {
  stopProgressTimer();
  progressTimerId = setInterval(() => {
    const elapsedSec = Math.floor((Date.now() - startedAtMs) / 1000);
    setStatus(
      [
        `Converting ${selectedInputPaths.length} file(s)...`,
        `Elapsed time: ${elapsedSec}s`,
        "The app is still running; this can take time on large files.",
      ].join("\n"),
    );
  }, 1000);
}

function renderSelectedInputs() {
  if (selectedInputPaths.length === 0) {
    inputListEl.value = "";
    inputListEl.placeholder = "No files selected";
    return;
  }

  inputListEl.value = selectedInputPaths.join("\n");
}

pickInputBtn.addEventListener("click", async () => {
  if (!invoke) {
    setStatus("Tauri bridge is not available.");
    return;
  }

  try {
    const picked = await invoke("pick_input_files");
    if (picked && picked.length > 0) {
      selectedInputPaths = picked;
      renderSelectedInputs();

      if (!outputDirEl.value.trim() && selectedInputPaths.length === 1) {
        const suggested = await invoke("suggest_output_dir", {
          inputPath: selectedInputPaths[0],
        });
        outputDirEl.value = suggested;
      }

      setStatus(`Selected ${selectedInputPaths.length} file(s).`);
    }
  } catch (error) {
    setStatus(`Failed to select input files: ${error}`);
  }
});

pickOutputBtn.addEventListener("click", async () => {
  if (!invoke) {
    setStatus("Tauri bridge is not available.");
    return;
  }

  try {
    const picked = await invoke("pick_output_dir");
    if (picked) {
      outputDirEl.value = picked;
      setStatus("Output folder selected.");
    }
  } catch (error) {
    setStatus(`Failed to select output folder: ${error}`);
  }
});

convertBtn.addEventListener("click", async () => {
  if (!invoke) {
    setStatus("Tauri bridge is not available.");
    return;
  }

  const dpi = Number(dpiEl.value);
  const outputDir = outputDirEl.value.trim();

  if (selectedInputPaths.length === 0) {
    setStatus("Please select at least one input file.");
    return;
  }

  if (!Number.isInteger(dpi) || dpi < 72 || dpi > 600) {
    setStatus("DPI must be an integer between 72 and 600.");
    return;
  }

  setBusy(true);
  resultTextEl.value = "";
  const startedAtMs = Date.now();
  setStatus(`Starting conversion for ${selectedInputPaths.length} file(s)...\nElapsed time: 0s`);
  startProgressTimer(startedAtMs);

  try {
    const result = await invoke("convert_documents", {
      inputPaths: selectedInputPaths,
      outputDir: outputDir || null,
      dpi,
    });

    const summary = [
      "Conversion finished",
      `Succeeded: ${result.success_count}`,
      `Failed: ${result.failed_count}`,
      `Elapsed time: ${(result.elapsed_ms / 1000).toFixed(2)}s`,
    ];

    if (outputDir) {
      summary.push(`Export base folder: ${outputDir}`);
    } else {
      summary.push("No output folder selected: showing text in-app only.");
    }

    setStatus(summary.join("\n"));

    if (result.combined_text && result.combined_text.trim()) {
      resultTextEl.value = result.combined_text;
    } else {
      const failureDetails = result.items
        .filter((item) => !item.success)
        .map((item) => `- ${item.input_path}: ${item.error ?? "Unknown error"}`)
        .join("\n");
      resultTextEl.value = failureDetails || "No text output generated.";
    }
  } catch (error) {
    setStatus(`Conversion failed:\n${error}`);
  } finally {
    stopProgressTimer();
    setBusy(false);
  }
});

copyBtn.addEventListener("click", async () => {
  const text = resultTextEl.value;
  if (!text.trim()) {
    setStatus("There is no text to copy.");
    return;
  }

  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    } else {
      resultTextEl.focus();
      resultTextEl.select();
      document.execCommand("copy");
      resultTextEl.setSelectionRange(resultTextEl.value.length, resultTextEl.value.length);
    }
    setStatus("Copied extracted text to clipboard.");
  } catch (error) {
    setStatus(`Failed to copy text: ${error}`);
  }
});

if (!invoke) {
  setStatus("This page must be run inside a Tauri app.");
}
