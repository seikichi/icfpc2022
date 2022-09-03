import { promises as fs } from 'fs'
import path from 'path'
import { useRef, useEffect } from "react"

interface Props {
    id: string
    pngPath: string
}

export async function getStaticPaths() {
    const dir = path.join(process.cwd(), '..', 'problems')
    const files = await fs.readdir(dir)

    return {
        paths: files.map(name => {
            const id = path.basename(name, '.png')
            return { params: { id } }
        }),
        fallback: false,
    }
}

export async function getStaticProps({ params: { id } }: { params: { id: string; } }) {
    const pngPath = path.join("..", 'problems', `${id}.png`);

    return { props: { id, pngPath } }
}

function generatePcutProgram(canvas: HTMLCanvasElement, blockIdObj:any, down: any, up: any) {
    const min_point = { x: Math.min(down.x, up.x, canvas.width -1 ), y: Math.min(down.y, up.y, canvas.height -1 ) }
    const max_point = { x: Math.max(down.x, up.x, 1), y: Math.max(down.y, up.y, 1) }
    let root = blockIdObj.root

    // decide color
    const ctx = canvas.getContext("2d")
    const width = max_point.x - min_point.x
    const height = max_point.y - min_point.y
    const data = ctx!.getImageData(min_point.x, min_point.y, width, height).data
    const sum = {r: 0, g: 0, b: 0, a: 0}
    for (let i=0; i< data.length; i+=4) {
        sum.r += data[i]
        sum.g += data[i+1]
        sum.b += data[i+2]
        sum.a += data[i+3]
    }
    sum.r = Math.round(sum.r / (width*height))
    sum.g = Math.round(sum.g / (width*height))
    sum.b = Math.round(sum.b / (width*height))
    sum.a = Math.round(sum.a / (width*height))

    // TODO: to WASM
    const commands = []
    commands.push(`cut [${root}] [${min_point.x}, ${canvas.height - min_point.y + 1}]\n`)
    commands.push(`cut [${root}.1] [${max_point.x}, ${canvas.height - max_point.y + 1}]\n`)
    commands.push(`color [${root}.1.3] [${sum.r}, ${sum.g}, ${sum.b}, ${sum.a}]\n`)
    commands.push(`merge [${root}.1.0] [${root}.1.1]\n`)
    commands.push(`merge [${root}.1.2] [${root}.1.3]\n`)
    commands.push(`merge [${root+1}] [${root+2}]\n`)
    commands.push(`merge [${root}.2] [${root+3}]\n`)
    commands.push(`merge [${root}.0] [${root}.3]\n`)
    commands.push(`merge [${root+4}] [${root+5}]\n`)
    blockIdObj.root += 6
    return commands
}

export default function Pcut({ id, pngPath }: Props) {
    const canvasRef = useRef(null as HTMLCanvasElement | null)
    const SIZE = 400;

    useEffect(() => {
        if (!canvasRef) {
            return;
        }
        const canvas = canvasRef.current;
        if (!canvas) {
            return
        }
        canvas.style.border = "4px solid";
        let down: any = null
        let up: any = null
        let blockIdObj = {root: 0}

        const commandsRef = document.getElementById("commands")
        console.log("canvas", canvas.width, canvas.height)
        canvas.onmousedown = (e) => {
            const rect = canvas.getBoundingClientRect()
            down = {
                x: e.clientX - rect.left,
                y: e.clientY - rect.top
            }
            console.log("down", down.x, down.y)
            if (down.x < 0 || down.x >= canvas.width || down.y < 0 || down.y >= canvas.height) {
                down = null
                console.log("abort: down")
                return
            }
        }
        canvas.onmouseup = (e) => {
            console.log(e)
            const rect = canvas.getBoundingClientRect()
            up = {
                x: e.clientX - rect.left,
                y: e.clientY - rect.top,
            }
            console.log("up", up.x, up.y)
            if (down == null || up.x < 0 || up.x >= canvas.width || up.y < 0 || up.y >= canvas.height) {
                down = null
                up = null
                console.log("abort: up")
                return
            }
            for (let st of generatePcutProgram(canvas, blockIdObj, down, up)) {
                commandsRef!.innerHTML += st
            }
        }

        console.log("pngPath", pngPath)
        const ctx = canvas.getContext('2d')!
        const img = document.createElement("img");
        img.src = pngPath
        img.onload = () => {
            ctx.drawImage(img, 0, 0, canvas.width, canvas.height)
        }
    }, [canvasRef, pngPath]);

    return (
        <div>
            <canvas ref={canvasRef} width={SIZE} height={SIZE}></canvas>
            <textarea id="commands"></textarea>
        </div>
    )
}
