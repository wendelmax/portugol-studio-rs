// Protocolo de quadros definido em crates/ferro-vm/src/graphics.rs.
const Op = {
  UploadImage: 1,
  ReleaseImage: 2,
  LoadFont: 3,
  Clear: 10,
  Color: 11,
  Gradient: 12,
  Font: 13,
  Rect: 20,
  Ellipse: 21,
  Line: 22,
  Point: 23,
  Polygon: 24,
  Text: 25,
  Image: 26,
  ImagePart: 27,
} as const;

class FrameReader {
  private readonly view: DataView;
  private offset = 0;
  private readonly decoder = new TextDecoder();

  constructor(private readonly frame: Uint8Array<ArrayBuffer>) {
    this.view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);
  }

  get done() { return this.offset === this.frame.byteLength; }
  u8() { return this.view.getUint8(this.offset++); }
  i32() {
    const value = this.view.getInt32(this.offset, true);
    this.offset += 4;
    return value;
  }
  u32() { return this.i32() >>> 0; }
  f32() {
    const value = this.view.getFloat32(this.offset, true);
    this.offset += 4;
    return value;
  }
  bytes(length: number) {
    if (length > this.frame.byteLength - this.offset) throw new Error("Quadro gráfico incompleto.");
    const value = this.frame.subarray(this.offset, this.offset + length);
    this.offset += length;
    return value;
  }
  text() { return this.decoder.decode(this.bytes(this.u32())); }
  rotation() { return { angle: this.f32(), x: this.i32(), y: this.i32() }; }
}

type Rotation = ReturnType<FrameReader["rotation"]>;
type Bounds = [x: number, y: number, width: number, height: number];

const gradientDirections = [
  [0, 0, 1, 0], [1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 0, 1],
  [0, 0, 1, 1], [1, 0, 0, 1], [0, 1, 1, 0], [1, 1, 0, 0],
];

function color(argb: number) {
  return `rgba(${(argb >>> 16) & 255}, ${(argb >>> 8) & 255}, ${argb & 255}, ${(argb >>> 24) / 255})`;
}

function surface(width: number, height: number) {
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  return canvas;
}

function context(canvas: HTMLCanvasElement) {
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Canvas 2D indisponível na janela gráfica.");
  return ctx;
}

export class GraphicsRenderer {
  private readonly images = new Map<number, HTMLCanvasElement>();
  private readonly registeredFonts = new Map<string, FontFace>();
  private color = "#000";
  private gradient: { direction: number; start: string; end: string } | null = null;
  private font = { family: "FerroSans", size: 12, style: 0 };

  constructor(private readonly canvas: HTMLCanvasElement) {}

  get size(): readonly [number, number] {
    return [this.canvas.width, this.canvas.height];
  }

  resize(width: number, height: number) {
    if (width === this.canvas.width && height === this.canvas.height) return;
    this.canvas.width = width;
    this.canvas.height = height;
  }

  async present(frame: Uint8Array<ArrayBuffer>) {
    const ctx = context(this.canvas);
    ctx.fillStyle = "#000";
    ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    await this.draw(ctx, frame);
  }

  async snapshot(frame: Uint8Array<ArrayBuffer>, width: number, height: number) {
    const ctx = context(surface(width, height));
    await this.draw(ctx, frame);
    return ctx.getImageData(0, 0, width, height).data;
  }

  private paint(ctx: CanvasRenderingContext2D, [x, y, width, height]: Bounds) {
    const { gradient } = this;
    if (!gradient) return this.color;
    const [x0, y0, x1, y1] = gradientDirections[gradient.direction] ?? gradientDirections[0];
    const fromX = x + x0 * width;
    const fromY = y + y0 * height;
    const toX = x + x1 * width;
    const toY = y + y1 * height;
    if (fromX === toX && fromY === toY) return gradient.start;
    const paint = ctx.createLinearGradient(fromX, fromY, toX, toY);
    paint.addColorStop(0, gradient.start);
    paint.addColorStop(1, gradient.end);
    return paint;
  }

  private drawRotated(ctx: CanvasRenderingContext2D, rotation: Rotation, draw: () => void) {
    ctx.save();
    try {
      if (rotation.angle) {
        ctx.translate(rotation.x, rotation.y);
        ctx.rotate(rotation.angle);
        ctx.translate(-rotation.x, -rotation.y);
      }
      draw();
    } finally {
      ctx.restore();
    }
  }

  private shape(
    ctx: CanvasRenderingContext2D, bounds: Bounds, rotation: Rotation, fill: boolean, path: () => void,
  ) {
    this.drawRotated(ctx, rotation, () => {
      ctx.beginPath();
      path();
      if (fill) {
        ctx.fillStyle = this.paint(ctx, bounds);
        ctx.fill();
      } else {
        ctx.strokeStyle = this.paint(ctx, bounds);
        ctx.stroke();
      }
    });
  }

  private async loadFont(frame: FrameReader) {
    const family = frame.text();
    const style = frame.u8();
    const bytes = frame.bytes(frame.u32());
    const font = await new FontFace(family, bytes, {
      style: style & 1 ? "italic" : "normal",
      weight: style & 2 ? "bold" : "normal",
    }).load();
    const key = family + ":" + style;
    const previous = this.registeredFonts.get(key);
    if (previous) document.fonts.delete(previous);
    document.fonts.add(font);
    this.registeredFonts.set(key, font);
  }

  private uploadImage(frame: FrameReader) {
    const id = frame.u32();
    const width = frame.u32();
    const height = frame.u32();
    const rgba = new Uint8ClampedArray(frame.bytes(width * height * 4));
    const image = surface(width, height);
    context(image).putImageData(new ImageData(rgba, width, height), 0, 0);
    this.images.set(id, image);
  }

  private drawImage(ctx: CanvasRenderingContext2D, frame: FrameReader, cropped: boolean) {
    const image = this.images.get(frame.u32());
    const x = frame.i32(), y = frame.i32();
    const crop = cropped ? [frame.i32(), frame.i32(), frame.i32(), frame.i32()] as const : null;
    const opacity = frame.u8() / 255;
    const rotation = frame.rotation();
    if (!image) throw new Error("Quadro gráfico referencia uma imagem indisponível.");
    this.drawRotated(ctx, rotation, () => {
      ctx.globalAlpha = opacity;
      if (crop) {
        const [sx, sy, width, height] = crop;
        ctx.drawImage(image, sx, sy, width, height, x, y, width, height);
      } else {
        ctx.drawImage(image, x, y);
      }
    });
  }

  private drawText(ctx: CanvasRenderingContext2D, frame: FrameReader) {
    const x = frame.i32(), y = frame.i32();
    const text = frame.text();
    this.drawRotated(ctx, frame.rotation(), () => {
      const { family, size, style } = this.font;
      ctx.font = `${style & 1 ? "italic" : "normal"} ${style & 2 ? "bold" : "normal"} ${size}px ${JSON.stringify(family)}, sans-serif`;
      const width = ctx.measureText(text).width;
      ctx.fillStyle = this.paint(ctx, [x, y - size, width, size]);
      ctx.fillText(text, x, y);
      if (style & 4) ctx.fillRect(x, y + 1, width, Math.max(1, size / 16));
    });
  }

  private async draw(ctx: CanvasRenderingContext2D, bytes: Uint8Array<ArrayBuffer>) {
    const frame = new FrameReader(bytes);
    while (!frame.done) {
      const op = frame.u8();
      switch (op) {
        case Op.UploadImage: this.uploadImage(frame); break;
        case Op.ReleaseImage: this.images.delete(frame.u32()); break;
        case Op.LoadFont: await this.loadFont(frame); break;
        case Op.Clear: {
          const width = frame.i32(), height = frame.i32();
          ctx.fillStyle = this.paint(ctx, [0, 0, width, height]);
          ctx.fillRect(0, 0, width, height);
          break;
        }
        case Op.Color:
          this.color = color(frame.u32());
          this.gradient = null;
          break;
        case Op.Gradient:
          this.gradient = { direction: frame.u8(), start: color(frame.u32()), end: color(frame.u32()) };
          break;
        case Op.Font:
          this.font = { family: frame.text(), size: frame.f32(), style: frame.u8() };
          break;
        case Op.Rect:
        case Op.Ellipse: {
          const bounds: Bounds = [frame.i32(), frame.i32(), frame.i32(), frame.i32()];
          const [x, y, width, height] = bounds;
          const rounded = op === Op.Rect && Boolean(frame.u8());
          const fill = Boolean(frame.u8());
          const rotation = frame.rotation();
          if (width < 0 || height < 0) break;
          this.shape(ctx, bounds, rotation, fill, () => {
            if (op === Op.Ellipse) {
              ctx.ellipse(x + width / 2, y + height / 2, width / 2, height / 2, 0, 0, 2 * Math.PI);
            } else if (rounded) {
              ctx.roundRect(x, y, width, height, Math.floor(Math.min(width, height) * 0.2) / 2);
            } else {
              ctx.rect(x, y, width, height);
            }
          });
          break;
        }
        case Op.Line: {
          const x = frame.i32(), y = frame.i32(), endX = frame.i32(), endY = frame.i32();
          this.shape(ctx, [x, y, endX - x, endY - y], frame.rotation(), false, () => {
            ctx.moveTo(x, y);
            ctx.lineTo(endX, endY);
          });
          break;
        }
        case Op.Point: {
          const x = frame.i32(), y = frame.i32();
          ctx.fillStyle = this.paint(ctx, [x, y, 1, 1]);
          ctx.fillRect(x, y, 1, 1);
          break;
        }
        case Op.Polygon: {
          const count = frame.u32();
          const coordinates = frame.bytes(count * 8);
          const points = new FrameReader(coordinates);
          const fill = Boolean(frame.u8());
          const rotation = frame.rotation();
          if (!count) break;
          this.drawRotated(ctx, rotation, () => {
            let left = Infinity, right = -Infinity, top = Infinity, bottom = -Infinity;
            ctx.beginPath();
            for (let index = 0; index < count; index++) {
              const x = points.i32(), y = points.i32();
              if (index === 0) ctx.moveTo(x, y);
              else ctx.lineTo(x, y);
              left = Math.min(left, x);
              right = Math.max(right, x);
              top = Math.min(top, y);
              bottom = Math.max(bottom, y);
            }
            ctx.closePath();
            const paint = this.paint(ctx, [left, top, right - left, bottom - top]);
            if (fill) {
              ctx.fillStyle = paint;
              ctx.fill();
            } else {
              ctx.strokeStyle = paint;
              ctx.stroke();
            }
          });
          break;
        }
        case Op.Text: this.drawText(ctx, frame); break;
        case Op.Image:
        case Op.ImagePart: this.drawImage(ctx, frame, op === Op.ImagePart); break;
        default: throw new Error(`Operação gráfica desconhecida: ${op}.`);
      }
    }
  }
}
