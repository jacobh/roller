import init, { run_app } from './pkg/roller_web_ui.js';
async function main() {
   await init('/pkg/roller_web_ui_bg.wasm');
   run_app();
}
main()
