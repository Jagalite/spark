#!/usr/bin/env python3
"""Fixed-font derivatives of actual Rust styled-cell captures. Requires Pillow.
Exact JSON remains authoritative; this is not a terminal-emulator recording.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
from PIL import Image, ImageDraw, ImageFont, PngImagePlugin, __version__

parser = argparse.ArgumentParser()
parser.add_argument('--font', default='/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf')
parser.add_argument('--design', default='all', help='stable design ID, or all available captures')
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
out = root / 'previews'
font = ImageFont.truetype(args.font, 22)
label_font = ImageFont.truetype(args.font, 15)
CW, CH, PAD = 15, 30, 30
BG = (10,17,27)
FG = (155,181,208)
COLORS = {
    'Black': (0,0,0), 'Red': (128,0,0), 'Green': (0,128,0),
    'Yellow': (128,128,0), 'Blue': (0,0,128), 'Magenta': (128,0,128),
    'Cyan': (0,128,128), 'Gray': (192,192,192), 'DarkGray': (128,128,128),
    'LightRed': (255,0,0), 'LightGreen': (0,255,0), 'LightYellow': (255,255,0),
    'LightBlue': (0,0,255), 'LightMagenta': (255,0,255), 'LightCyan': (0,255,255),
    'White': (255,255,255),
}

def color(value, background=False):
    if value == 'Reset':
        return BG if background else FG
    match = re.fullmatch(r'Rgb\((\d+), (\d+), (\d+)\)', value)
    if match:
        return tuple(map(int, match.groups()))
    indexed = re.fullmatch(r'Indexed\((\d+)\)', value)
    if indexed:
        index = int(indexed[1])
        if index < 16:
            return list(COLORS.values())[index]
        if index >= 232:
            return (8+(index-232)*10,)*3
        index -= 16
        level = lambda n: 0 if n == 0 else 55+n*40
        return level(index//36), level((index//6)%6), level(index%6)
    if value not in COLORS:
        raise ValueError(f'Unsupported captured color {value}')
    return COLORS[value]

def cell_colors(cell):
    fg, bg = color(cell['fg']), color(cell['bg'], True)
    if cell['modifiers'] & 64:
        fg, bg = bg, fg
    if cell['modifiers'] & 2:
        fg = tuple(int(c*0.65) for c in fg)
    return fg, bg

def paint(canvas, cells, x, y):
    draw = ImageDraw.Draw(canvas)
    # Paint all backgrounds before symbols so host wide glyphs aren't clipped by
    # the continuation cell's background. Font support is a separate limitation.
    for row, contents in enumerate(cells['rows']):
        for col, cell in enumerate(contents):
            px, py = x+col*CW, y+row*CH
            _, bg = cell_colors(cell)
            draw.rectangle((px,py,px+CW-1,py+CH-1), fill=bg)
    for row, contents in enumerate(cells['rows']):
        for col, cell in enumerate(contents):
            px, py = x+col*CW, y+row*CH
            fg, _ = cell_colors(cell)
            if not (cell['modifiers'] & 128):
                draw.text((px,py+2), cell['symbol'], font=font, fill=fg)
                if cell['modifiers'] & 1:
                    draw.text((px+0.5,py+2), cell['symbol'], font=font, fill=fg)
            if cell['modifiers'] & 8:
                underline = fg if cell.get('underline_color','Reset') == 'Reset' else color(cell['underline_color'])
                draw.line((px,py+CH-4,px+CW-1,py+CH-4),fill=underline)
            if cell['modifiers'] & 256:
                draw.line((px,py+CH//2,px+CW-1,py+CH//2),fill=fg)

def tile(cells, label):
    width, height = cells['area'][2:]
    image = Image.new('RGB',(max(width*CW+2*PAD,600),height*CH+2*PAD+35),BG)
    ImageDraw.Draw(image).text((PAD,18),label,font=label_font,fill=FG)
    paint(image,cells,(image.width-width*CW)//2,55)
    return image

def render(source):
    data = json.loads(source.read_text())
    design = data['design']
    reference = data.get('reference', 'R01')
    kind = data.get('playback_kind','loop')
    duration = data.get('frame_duration_ms',40)
    frames = [tile(frame['cells'], f"{reference} / {design.upper()} / {frame['time_ms']:04d} ms") for frame in data['frames']]
    for suffix, factor in [('',1),('-slow',4)]:
        frames[0].save(out/f'{design}{suffix}.gif', save_all=True, append_images=frames[1:], duration=duration*factor, loop=0, disposal=2)
    selected = sorted({0,len(frames)//8,len(frames)//4,len(frames)//2,3*len(frames)//4,len(frames)-1})
    tile_h = frames[0].height
    sheet = Image.new('RGB',(max(frames[0].width,780),tile_h*len(selected)+120),BG)
    for i,index in enumerate(selected):
        sheet.paste(frames[index],((sheet.width-frames[index].width)//2,i*tile_h))
    draw = ImageDraw.Draw(sheet)
    seed = data['seed'] if data['seed'] is not None else 'not applicable'
    draw.text((PAD,tile_h*len(selected)+15),f"Actual Rust cells / seed {seed} / {kind} / visual revision {data['visual_revision']}",font=label_font,fill=FG)
    draw.text((PAD,tile_h*len(selected)+45),'Fixed font, offscreen capture; not terminal-emulator qualification',font=label_font,fill=FG)
    digest = hashlib.sha256(source.read_bytes()).hexdigest()
    metadata = PngImagePlugin.PngInfo()
    metadata.add_text('Description',f"Rust capture SHA256 {digest}; source {data['source_sha256']}")
    sheet.save(out/f'{design}-contact-sheet.png',pnginfo=metadata)
    tile(data['embedded'],f'{reference} / SYNTHETIC REQUEST / RESULT').save(out/f'{design}-embedded.png',pnginfo=metadata)
    tile(data['fallback'],f'{reference} / QUIET ASCII / MONOCHROME / STATIC').save(out/f'{design}-fallback.png',pnginfo=metadata)
    if data.get('minimum_full'):
        tile(data['minimum_full'],f'{reference} / DECLARED MINIMUM FULL PRESENTATION').save(out/f'{design}-minimum-full.png',pnginfo=metadata)
    if data.get('smallest'):
        tile(data['smallest'],f'{reference} / 3x1 QUIET FALLBACK').save(out/f'{design}-smallest.png',pnginfo=metadata)
    if data.get('boundaries'):
        boundary_tiles = [tile(frame['cells'],f"{reference} / BOUNDARY / {frame['time_ms']} ms") for frame in data['boundaries']]
        boundary_sheet = Image.new('RGB',(max(t.width for t in boundary_tiles),sum(t.height for t in boundary_tiles)),BG)
        y = 0
        for item in boundary_tiles:
            boundary_sheet.paste(item,(0,y)); y += item.height
        boundary_sheet.save(out/f'{design}-boundaries.png',pnginfo=metadata)
    for name,cells in data.get('embeddings',{}).items():
        tile(cells,f'{reference} / {name.upper()} / SYNTHETIC HOST').save(out/f'{design}-{name}.png',pnginfo=metadata)
    if data.get('unknown_progress'):
        tile(data['unknown_progress'],f'{reference} / CALLER PROGRESS UNKNOWN').save(out/f'{design}-unknown.png',pnginfo=metadata)
    # Keep existing R01 document links working; its exact fixtures remain separate.
    if design == 'segmented-comet':
        sheet.save(out/'contact-sheet.png',pnginfo=metadata)
        tile(data['embedded'],'SPARK / SYNTHETIC APPLICATION EMBEDDING').save(out/'embedded.png',pnginfo=metadata)
        tile(data['fallback'],'SPARK / QUIET ASCII + MONOCHROME + STATIC').save(out/'fallback.png',pnginfo=metadata)
    report = {
        'schema':2,'design':design,'source_sha256':data['source_sha256'],'capture_sha256':digest,
        'renderer':'scripts/render_previews.py','renderer_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'pillow':__version__,'font':Path(args.font).name,'font_sha256':hashlib.sha256(Path(args.font).read_bytes()).hexdigest(),
        'cell_pixels':[CW,CH],'background':'#0a111b','frame_duration_ms':duration,'frames':len(frames),'playback_kind':kind,
        'limitations':['Fixed-font offscreen preview, not a real terminal recording','BOLD and DIM are approximated; italics and blink are not simulated','Unsupported font glyphs may differ from terminal fonts; exact JSON remains authoritative','GIFs replay observation windows / one-shots where labeled; this does not imply a seamless loop']
    }
    (out/f'{design}.render.json').write_text(json.dumps(report,indent=2)+'\n')
    # Poster uses a true sampled mid-sequence cell frame, not promotional artwork.
    poster = tile(data['frames'][len(frames)//2]['cells'],f'{reference} / {data.get("title",design)}')
    return report, poster

sources = sorted(out.glob('*.cells.json')) if args.design=='all' else [out/f'{args.design}.cells.json']
if not sources:
    raise SystemExit('No Rust captures found; run cargo xtask render all first')
reports, posters = [], []
for source in sources:
    report, poster = render(source)
    reports.append(report)
    posters.append((report['design'],poster))
    print(f"Rendered {report['design']}: {report['frames']} actual frames, slow replay, boundaries, embeddings, quiet fallback")
if args.design == 'all':
    order = ['segmented-comet','assembly-progress','border-runner','telemetry-embers','pocket-mascot','prism-reveal']
    posters.sort(key=lambda item:order.index(item[0]))
    card_w = max(p.width for _,p in posters)
    row_heights = [max(p.height for _,p in posters[i:i+2])+25 for i in range(0,len(posters),2)]
    sheet = Image.new('RGB',(card_w*2,sum(row_heights)+90),BG)
    for i,(_,poster) in enumerate(posters):
        sheet.paste(poster,((i%2)*card_w+(card_w-poster.width)//2,sum(row_heights[:i//2])))
    ImageDraw.Draw(sheet).text((PAD,sheet.height-65),'SPARK / SIX ACTUAL RUST SAMPLES / local visual candidates',font=label_font,fill=FG)
    ImageDraw.Draw(sheet).text((PAD,sheet.height-35),'Fixed DejaVu Sans Mono preview; see exact cells and recorded limits',font=label_font,fill=FG)
    sheet.save(out/'catalog-contact-sheet.png')
    (out/'render-manifest.json').write_text(json.dumps({'schema':2,'captures':reports},indent=2)+'\n')
