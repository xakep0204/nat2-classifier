// Wait for Tauri to be ready
async function waitForTauri() {
  return new Promise((resolve) => {
    if (window.__TAURI__) {
      resolve();
    } else {
      const check = () => {
        if (window.__TAURI__) {
          resolve();
        } else {
          setTimeout(check, 100);
        }
      };
      check();
    }
  });
}

let invoke, open;

async function initTauri() {
  await waitForTauri();
  invoke = window.__TAURI__.core.invoke;
  open = window.__TAURI__.dialog.open;
  console.log("Tauri APIs initialized:", { invoke, open });
}

async function openFile() {
  try {
    console.log("Opening file dialog...");
    if (!open) {
      console.error("Open function not available");
      updateFileStatus("File dialog not available", 'error');
      return;
    }

    const filePath = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: 'Supported Files',
          extensions: ['fasta', 'csv']
        }
      ]
    });

    console.log("File path selected:", filePath);
    if (filePath) {
      const fileName = typeof filePath === 'string' ? filePath.split('/').pop() : filePath;
      await handleFileSelected(filePath, fileName);
    } else {
      console.log("No file selected");
    }
  } catch (error) {
    console.error("Error opening file:", error);
    updateFileStatus(`Error opening file: ${error}`, 'error');
  }
}

let selectedFilePath = null;
let selectedFileName = null;

async function handleFileSelected(filePath, fileName) {
  try {
    console.log("Reading file:", filePath);
    
    // Check file extension
    const allowedExtensions = ['.fasta', '.csv'];
    const fileExtension = fileName.toLowerCase().substring(fileName.lastIndexOf('.'));
    
    if (!allowedExtensions.includes(fileExtension)) {
      updateFileStatus(`✗ Invalid file type. Only .fasta and .csv files are allowed.`, 'error');
      return;
    }
    
    if (!invoke) {
      console.error("Invoke function not available");
      updateFileStatus("Invoke not available", 'error');
      return;
    }

    const firstTwo = await invoke("read_file_start", { path: filePath });
    const fileListEl = document.querySelector("#file-list");
    fileListEl.textContent = `Selected: ${fileName} | First 2 chars: "${firstTwo}"`;
    updateFileStatus(`✓ File loaded: ${fileName}`, 'success');

    // Store file info and show analyze button
    selectedFilePath = filePath;
    selectedFileName = fileName;
    document.querySelector("#analyze-btn").classList.remove("hidden");

    console.log("Selected file:", filePath);
  } catch (error) {
    console.error("Error reading file:", error);
    updateFileStatus(`✗ Error reading file: ${error}`, 'error');
  }
}

function updateFileStatus(message, type = 'info') {
  const statusEl = document.querySelector("#file-status");
  if (statusEl) {
    statusEl.textContent = message;
    statusEl.className = `status-message ${type}`;
  } else {
    console.warn("Status element not found");
  }
}

function switchToPage(pageId) {
  // Hide all pages
  document.querySelectorAll('.page').forEach(page => {
    page.classList.remove('active');
  });

  // Show target page
  document.getElementById(pageId).classList.add('active');
}

async function startAnalysis() {
  if (!selectedFilePath) {
    updateFileStatus("No file selected", 'error');
    return;
  }

  // Switch to analysis page
  switchToPage('analysis-page');

  // Show loading
  document.getElementById('loading-container').classList.remove('hidden');
  document.getElementById('result-container').classList.add('hidden');

  try {
    console.log("Starting NAT2 analysis for:", selectedFilePath);
    const result = await invoke("analyze_nat2", { path: selectedFilePath });
    console.log("Analysis result:", result);

    // Random loading time between 0.5-1 seconds
    const loadingTime = Math.random() * 500 + 500; // 500-1000ms

    setTimeout(() => {
      showResults(result);
    }, loadingTime);
  } catch (error) {
    console.error("Analysis error:", error);
    // Random loading time even for errors
    const loadingTime = Math.random() * 500 + 500;

    setTimeout(() => {
      showResults(`Error during analysis: ${error}`);
    }, loadingTime);
  }
}

function showResults(result) {
  // Hide loading, show results
  document.getElementById('loading-container').classList.add('hidden');
  document.getElementById('result-container').classList.remove('hidden');

  // Display the classification result
  document.getElementById('classification-result').textContent = result;
}

function goBack() {
  // Reset file selection
  selectedFilePath = null;
  selectedFileName = null;
  document.querySelector("#analyze-btn").classList.add("hidden");
  document.querySelector("#file-list").textContent = "";
  updateFileStatus("", 'info');

  // Switch back to file page
  switchToPage('file-page');
}

window.addEventListener("DOMContentLoaded", async () => {
  console.log("DOM Content Loaded");

  // Initialize Tauri
  await initTauri();

  // File picker button
  const openFileBtn = document.querySelector("#open-file-btn");
  if (openFileBtn) {
    console.log("Button found, attaching click listener");
    openFileBtn.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      console.log("Button clicked");
      openFile();
    });
  } else {
    console.error("Button not found");
  }

  // Analyze button
  const analyzeBtn = document.querySelector("#analyze-btn");
  if (analyzeBtn) {
    analyzeBtn.addEventListener("click", startAnalysis);
  }

  // Back button
  const backBtn = document.querySelector("#back-btn");
  if (backBtn) {
    backBtn.addEventListener("click", goBack);
  }

  // Drag and drop setup
  const dropZone = document.querySelector("#drop-zone");
  if (dropZone) {
    console.log("Drop zone found, attaching listeners");
    dropZone.addEventListener("dragover", (e) => {
      e.preventDefault();
      e.stopPropagation();
      dropZone.classList.add("dragover");
    });

    dropZone.addEventListener("dragleave", (e) => {
      e.preventDefault();
      e.stopPropagation();
      dropZone.classList.remove("dragover");
    });

    dropZone.addEventListener("drop", async (e) => {
      e.preventDefault();
      e.stopPropagation();
      dropZone.classList.remove("dragover");

      const files = e.dataTransfer.files;
      if (files.length > 0) {
        const file = files[0];
        
        // Check file extension for drag-and-drop
        const allowedExtensions = ['.fasta', '.csv'];
        const fileExtension = file.name.toLowerCase().substring(file.name.lastIndexOf('.'));
        
        if (!allowedExtensions.includes(fileExtension)) {
          updateFileStatus(`✗ Invalid file type. Only .fasta and .csv files are allowed.`, 'error');
          return;
        }
        
        const filePath = file.path || file.name;
        console.log("File dropped:", file.name);
        await handleFileSelected(filePath, file.name);
      }
    });

    // Allow clicking on drop zone to open file picker
    dropZone.addEventListener("click", openFile);
  } else {
    console.error("Drop zone not found");
  }
});
