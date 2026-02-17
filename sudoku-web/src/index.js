function wasm_handler(wasm) {
  const {sudoku_ptr, sudoku_gen, value_to_char, memory} = wasm.instance.exports;
  const ptr = sudoku_ptr();

  function createGrid(size, values) {
    const table = document.createElement("table");
    for (let y = 0; y < size * size; y++) {
      const tr = document.createElement("tr");
      for (let x = 0; x < size * size; x++) {
        const td = document.createElement("td");
        let value = values[y * size * size + x]
        if (value != 255) {
          td.textContent = String.fromCodePoint(value_to_char(value));
        }
        if (x % size == size - 1) {
          td.classList.add("delimiter");
        }
        tr.append(td);
      }
      if (y % size == size - 1) {
        tr.classList.add("delimiter");
      }
      table.append(tr);
    }
    return table;
  }
  function randomSeed() {
    return Math.floor(Math.random() * 1000000000)
  }

  const validSizes = new Set([0, 1, 2, 3, 4, 5, 6, 7, 8]);

  function generateHandler() {
    const container = document.getElementById("grid-container");
    container.innerHTML = "";

    let size = document.getElementById("grid-size").value;
    let seed = document.getElementById("seed").value;

    size = parseInt(size);
    seed = seed ? parseInt(seed) : randomSeed();

    if (!validSizes.has(size) || !(seed >= 0 && seed < 1000000000)) return;

    let i = 1;
    while (true) {
      let status = document.getElementById("status");
      status.textContent = "generating attempt " + i + " with seed " + seed;
      let result = sudoku_gen(size, seed);
      if (result == 1) {
        break;
      }
      else if (result == 0) {
        i += 1;
        seed += 1;
      }
      else {
        status.textContent = "invalid size";
        break;
      }
    }

    const grid = new Uint8Array(memory.buffer, ptr, size * size * size * size);
    console.log(grid);
    container.appendChild(createGrid(size, grid));
  }

  const button = document.getElementById("gen-button");
  button.removeAttribute("disabled");
  button.onclick = generateHandler;
}

WebAssembly.instantiateStreaming(fetch("./sudoku.wasm"), {}).then(wasm_handler)


