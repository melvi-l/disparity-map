import './style.css'
import init, { greet } from "./wasm/disparity_map.js"

await init()

console.log(greet("main"))
