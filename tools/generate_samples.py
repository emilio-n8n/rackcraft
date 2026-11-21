#!/usr/bin/env python3
"""Generate the offline RackCraft Visio stencil fixtures.

The output files are ordinary .vssx (OPC/ZIP + Visio 2013 XML) packages with
masters, vector geometry, connection points, shape data and ICO previews. They
are intentionally small and deterministic so tests can assert exact inventory
counts and the application works without network access.

Usage:
    python3 tools/generate_samples.py
"""

from __future__ import annotations

import base64
import io
import struct
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SAMPLES = ROOT / "samples"
MM_PER_INCH = 25.4
RACK_U = 44.45
RACK_WIDTH = 482.6

NS = "http://schemas.microsoft.com/office/visio/2012/main"
RNS = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PKG_RNS = "http://schemas.openxmlformats.org/package/2006/relationships"


def mm(value: float) -> float:
    return value / MM_PER_INCH


def xml_escape(value: str) -> str:
    return (
        value.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&apos;")
    )


def icon_b64(color: tuple[int, int, int], kind: str) -> str:
    """Create a 32x32 32-bpp ICO with a simple recognizable glyph."""
    width = height = 32
    pixels = []
    for y in range(height):
        for x in range(width):
            r, g, b, a = 0, 0, 0, 0
            if 2 <= x < 30 and 2 <= y < 30:
                r, g, b, a = 26, 30, 38, 255
                if kind == "rack":
                    if x in (5, 26) or y in (4, 27):
                        r, g, b = color
                    elif x % 5 == 0 and 7 <= y <= 24:
                        r, g, b = 92, 104, 124
                elif kind in ("panel", "modular"):
                    if 5 <= x <= 26 and 10 <= y <= 21:
                        r, g, b = 10, 13, 18
                        if x % 4 == 0:
                            r, g, b = color
                elif kind == "switch":
                    if 4 <= x <= 27 and 12 <= y <= 20:
                        r, g, b = 10, 13, 18
                        if y == 13 and x % 3 == 0:
                            r, g, b = 84, 209, 132
                elif kind == "cable":
                    if 7 <= x <= 24 and 9 <= y <= 23:
                        r, g, b = 10, 13, 18
                        if x % 4 == 0:
                            r, g, b = color
                elif kind == "keystone":
                    if 8 <= x <= 23 and 5 <= y <= 26:
                        r, g, b = 235, 239, 246
                    if 11 <= x <= 20 and 12 <= y <= 20:
                        r, g, b = 10, 13, 18
                elif kind == "cloud":
                    if (8 <= x <= 24 and 12 <= y <= 24) or (15 <= x <= 27 and 8 <= y <= 20) or (18 <= x <= 31 and 13 <= y <= 25):
                        r, g, b = 210, 220, 235
                elif kind == "firewall":
                    if 6 <= x <= 27 and 8 <= y <= 25:
                        if (y // 5) % 2 == 0 and x % 7 in (1, 2):
                            r, g, b = 232, 87, 63
                        elif (y // 5) % 2 == 1 and x % 7 in (4, 5):
                            r, g, b = 232, 87, 63
                elif kind == "router":
                    if 8 <= x <= 25 and 13 <= y <= 21:
                        r, g, b = 92, 104, 124
                    if (10 <= x <= 13 and 15 <= y <= 18) or (5 <= x <= 30 and 15 <= y <= 17):
                        r, g, b = 232, 87, 63
                elif kind == "server":
                    for sy in (8, 15, 22):
                        if 8 <= x <= 25 and sy <= y <= sy + 4:
                            r, g, b = 210, 220, 235
                elif kind == "workstation":
                    if 6 <= x <= 26 and 7 <= y <= 20:
                        r, g, b = 210, 220, 235
                    if 13 <= x <= 19 and 21 <= y <= 25:
                        r, g, b = 92, 104, 124
            pixels.append((b, g, r, a))

    # BMP rows are bottom-up BGRA.
    raw = bytearray()
    for y in range(height - 1, -1, -1):
        for x in range(width):
            b, g, r, a = pixels[y * width + x]
            raw.extend((b, g, r, a))
    mask = bytes(width * height // 8)
    header = struct.pack("<IiiHHIIiiII", 40, width, height * 2, 1, 32, 0, len(raw), 0, 0, 0, 0)
    image = header + bytes(raw) + mask
    ico = struct.pack("<HHH", 0, 1, 1)
    ico += struct.pack("<BBBBHHII", width, height, 0, 0, 1, 32, len(image), 22)
    return base64.b64encode(ico + image).decode("ascii")


def cell(name: str, value: str) -> str:
    return f"<Cell N='{name}' V='{xml_escape(str(value))}'/>"


def geometry_rect(x: float, y: float, w: float, h: float, shape_h: float) -> str:
    """Rectangle in top-down shape millimetres -> Visio local (Y-up) inches."""
    x1, y1 = mm(x), mm(shape_h - y)
    x2, y2 = mm(x + w), mm(shape_h - y - h)
    return (
        "<Section N='Geometry'>"
        "<Cell N='NoFill' V='0'/><Cell N='NoLine' V='0'/>"
        f"<Row T='MoveTo' IX='1'>{cell('X', x1)}{cell('Y', y1)}</Row>"
        f"<Row T='LineTo' IX='2'>{cell('X', x2)}{cell('Y', y1)}</Row>"
        f"<Row T='LineTo' IX='3'>{cell('X', x2)}{cell('Y', y2)}</Row>"
        f"<Row T='LineTo' IX='4'>{cell('X', x1)}{cell('Y', y2)}</Row>"
        f"<Row T='LineTo' IX='5'>{cell('X', x1)}{cell('Y', y1)}</Row>"
        "</Section>"
    )


def geometry_line(x1: float, y1: float, x2: float, y2: float, shape_h: float) -> str:
    return (
        "<Section N='Geometry'>"
        "<Cell N='NoFill' V='1'/><Cell N='NoLine' V='0'/>"
        f"<Row T='MoveTo' IX='1'>{cell('X', mm(x1))}{cell('Y', mm(shape_h - y1))}</Row>"
        f"<Row T='LineTo' IX='2'>{cell('X', mm(x2))}{cell('Y', mm(shape_h - y2))}</Row>"
        "</Section>"
    )


def shape_xml(
    *,
    page_w: float,
    page_h: float,
    x: float,
    y: float,
    w: float,
    h: float,
    fill: str | None = "#2b3140",
    line: str | None = "#5e6a80",
    line_weight: float = 0.3,
    rects: list[tuple[float, float, float, float]] | None = None,
    lines: list[tuple[float, float, float, float]] | None = None,
    properties: list[tuple[str, str]] | None = None,
    connections: list[tuple[str, float, float]] | None = None,
    name: str = "Shape",
) -> str:
    """Top-level Visio shape spanning page coordinates (mm), Y-up conversion."""
    pin_x = mm(x + w / 2)
    pin_y = mm(page_h - (y + h / 2))
    stable_id = 100 + sum(ord(character) for character in name) % 800
    attrs = f"ID='{stable_id}' NameU='{xml_escape(name)}' Type='Shape'"
    cells = [
        cell("PinX", pin_x),
        cell("PinY", pin_y),
        cell("Width", mm(w)),
        cell("Height", mm(h)),
        cell("LocPinX", mm(w / 2)),
        cell("LocPinY", mm(h / 2)),
        cell("Angle", 0),
        cell("FlipX", 0),
        cell("FlipY", 0),
    ]
    sections = []
    if fill is not None:
        sections.append(f"<Section N='Fill'>{cell('FillForegnd', fill)}{cell('FillPattern', 1)}</Section>")
    if line is not None:
        sections.append(
            f"<Section N='Line'>{cell('LineColor', line)}{cell('LineWeight', mm(line_weight))}</Section>"
        )
    for rx, ry, rw, rh in rects or []:
        sections.append(geometry_rect(rx, ry, rw, rh, h))
    for x1, y1, x2, y2 in lines or []:
        sections.append(geometry_line(x1, y1, x2, y2, h))
    if properties:
        rows = []
        for index, (label, value) in enumerate(properties, start=1):
            rows.append(
                f"<Row N='Row_{index}'>{cell('Label', label)}{cell('Value', value)}"
                f"{cell('Prompt', '')}{cell('Type', 0)}</Row>"
            )
        sections.append("<Section N='Property'>" + "".join(rows) + "</Section>")
    if connections:
        rows = []
        for index, (conn_name, nx, ny) in enumerate(connections):
            # Main shapes span the whole page: local X = nx*W, local Y = (1-ny)*H.
            rows.append(
                f"<Row IX='{index}'>"
                f"{cell('X', mm(nx * w))}{cell('Y', mm((1.0 - ny) * h))}"
                f"{cell('DirX', 0)}{cell('DirY', 0)}{cell('Type', 0)}"
                f"{cell('Prompt', conn_name)}</Row>"
            )
        sections.append("<Section N='Connection'>" + "".join(rows) + "</Section>")
    return f"<Shape {attrs}>{''.join(cells)}{''.join(sections)}</Shape>"


def master_xml(master: dict) -> str:
    return (
        "<?xml version='1.0' encoding='utf-8' ?>"
        f"<MasterContents xmlns='{NS}' xmlns:r='{RNS}' xml:space='preserve'><Shapes>"
        + "".join(master["shapes"])
        + "</Shapes></MasterContents>"
    )


def write_vssx(path: Path, title: str, masters: list[dict]) -> None:
    masters_entries = []
    rels = []
    overrides = [
        '<Override PartName="/visio/document.xml" ContentType="application/vnd.ms-visio.stencil.main+xml"/>',
        '<Override PartName="/visio/masters/masters.xml" ContentType="application/vnd.ms-visio.masters+xml"/>',
    ]
    for index, master in enumerate(masters, start=1):
        master_id = index
        masters_entries.append(
            f"<Master ID='{master_id}' NameU='{xml_escape(master['name'])}' "
            f"IsCustomNameU='1' Name='{xml_escape(master['name'])}' IsCustomName='1' "
            "Prompt='' IconSize='1' AlignName='2' MatchByName='0' IconUpdate='1' "
            f"Hidden='0' MasterType='2'>"
            "<PageSheet LineStyle='0' FillStyle='0' TextStyle='0'>"
            f"{cell('PageWidth', mm(master['width_mm']))}"
            f"{cell('PageHeight', mm(master['height_mm']))}"
            f"{cell('PageScale', 1)}{cell('DrawingScale', 1)}"
            "<Cell N='DrawingSizeType' V='4'/><Cell N='DrawingScaleType' V='0'/>"
            "</PageSheet>"
            f"<Icon>{master['icon']}</Icon>"
            f"<Rel r:id='rId{master_id}'/>"
            "</Master>"
        )
        rels.append(
            f"<Relationship Id='rId{master_id}' "
            "Type='http://schemas.microsoft.com/visio/2010/relationships/master' "
            f"Target='master{master_id}.xml'/>"
        )
        overrides.append(
            f'<Override PartName="/visio/masters/master{master_id}.xml" '
            'ContentType="application/vnd.ms-visio.master+xml"/>'
        )

    content_types = (
        "<?xml version='1.0' encoding='UTF-8' standalone='yes'?>"
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        '<Default Extension="xml" ContentType="application/xml"/>'
        + "".join(overrides)
        + "</Types>"
    )
    root_rels = (
        "<?xml version='1.0' encoding='UTF-8' standalone='yes'?>"
        f'<Relationships xmlns="{PKG_RNS}">'
        '<Relationship Id="rId1" Type="http://schemas.microsoft.com/visio/2010/relationships/document" '
        'Target="visio/document.xml"/>'
        '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" '
        'Target="docProps/app.xml"/>'
        "</Relationships>"
    )
    document_xml = (
        "<?xml version='1.0' encoding='utf-8' ?>"
        f"<VisioDocument xmlns='{NS}' xmlns:r='{RNS}' xml:space='preserve'>"
        "<DocumentSettings TopPage='0' DefaultTextStyle='3' DefaultLineStyle='3' "
        "DefaultFillStyle='3' DefaultGuideStyle='4'>"
        "<GlueSettings>9</GlueSettings><SnapSettings>65847</SnapSettings>"
        "<DynamicGridEnabled>1</DynamicGridEnabled></DocumentSettings>"
        "<Colors><ColorEntry IX='0' RGB='#000000'/><ColorEntry IX='1' RGB='#FFFFFF'/></Colors>"
        "<FaceNames><FaceName NameU='Calibri' UnicodeRanges='-469750017 -1073732485 9 0' "
        "CharSets='536871423 0' Panose='2 15 5 2 2 2 4 3 2 4' Flags='325'/></FaceNames>"
        "<StyleSheets><StyleSheet ID='0' NameU='No Style' IsCustomNameU='1' Name='No Style' "
        "IsCustomName='1'><Cell N='EnableLineProps' V='1'/><Cell N='EnableFillProps' V='1'/>"
        "<Cell N='EnableTextProps' V='1'/><Cell N='LineWeight' V='0.01041666666666667'/>"
        "<Cell N='LineColor' V='#5e6a80'/><Cell N='LinePattern' V='1'/>"
        "<Cell N='FillForegnd' V='#2b3140'/><Cell N='FillPattern' V='1'/>"
        "</StyleSheet></StyleSheets>"
        "<Masters><Master ID='0' NameU='No Master' IsCustomNameU='1' Name='No Master' "
        "IsCustomName='1' Hidden='1' MasterType='0'/></Masters>"
        "</VisioDocument>"
    )
    masters_xml = (
        "<?xml version='1.0' encoding='utf-8' ?>"
        f"<Masters xmlns='{NS}' xmlns:r='{RNS}' xml:space='preserve'>"
        + "".join(masters_entries)
        + "</Masters>"
    )
    app_xml = (
        "<?xml version='1.0' encoding='UTF-8' standalone='yes'?>"
        '<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" '
        'xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">'
        f"<Application>RackCraft sample generator</Application><Company>RackCraft contributors</Company>"
        f"<TitlesOfParts><vt:vector size='{len(masters)}' baseType='lpstr'>"
        + "".join(f"<vt:lpstr>{xml_escape(master['name'])}</vt:lpstr>" for master in masters)
        + "</vt:vector></TitlesOfParts></Properties>"
    )

    parts = {
        "[Content_Types].xml": content_types,
        "_rels/.rels": root_rels,
        "docProps/app.xml": app_xml,
        "visio/document.xml": document_xml,
        "visio/masters/masters.xml": masters_xml,
        "visio/masters/_rels/masters.xml.rels": (
            "<?xml version='1.0' encoding='UTF-8' standalone='yes'?>"
            f'<Relationships xmlns="{PKG_RNS}">' + "".join(rels) + "</Relationships>"
        ),
    }
    for index, master in enumerate(masters, start=1):
        parts[f"visio/masters/master{index}.xml"] = master_xml(master)

    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as package:
        for name, content in parts.items():
            info = zipfile.ZipInfo(name, date_time=(2026, 11, 21, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            package.writestr(info, content)
    print(f"wrote {path} ({path.stat().st_size} bytes, {len(masters)} masters)")


def rack_master() -> dict:
    w, h = 600.0, 1181.25
    mount_x, mount_top = 58.7, 35.0
    shapes = []
    shapes.append(
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#1e222b",
            line="#5e6a80",
            line_weight=0.4,
            rects=[(0, 0, w, h), (mount_x, mount_top, RACK_WIDTH, 25 * RACK_U)],
            properties=[
                ("Manufacturer", "RackCraft"),
                ("Part Number", "RC-RK-25U"),
                ("Product Description", "25U 19-inch open frame rack, 600 x 1181 mm"),
                ("Category", "Racks & cabinets"),
                ("Rack Units", "25"),
                ("Mount Left", "58.7"),
                ("Mount Right", "58.7"),
                ("Mount Top", "35"),
                ("Mount Bottom", "35"),
            ],
            connections=[
                (f"U{25 - i}", 0.5, (mount_top + (i + 0.5) * RACK_U) / h)
                for i in range(25)
            ],
            name="Rack frame",
        )
    )
    # Inner opening and unit ticks, drawn as a line-only shape.
    shapes.append(
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill=None,
            line="#39414f",
            line_weight=0.2,
            lines=[(mount_x, mount_top + i * RACK_U, mount_x + RACK_WIDTH, mount_top + i * RACK_U) for i in range(26)],
            name="Rack unit guide",
        )
    )
    shapes.append(
        shape_xml(
            page_w=w,
            page_h=h,
            x=12,
            y=10,
            w=180,
            h=14,
            fill="#e8573f",
            line=None,
            rects=[(0, 0, 180, 14)],
            name="Name plate",
        )
    )
    return {
        "name": "RackCraft 25U Open Frame Rack",
        "width_mm": w,
        "height_mm": h,
        "icon": icon_b64((232, 87, 63), "rack"),
        "shapes": shapes,
    }


def patch_panel_master() -> dict:
    w, h = RACK_WIDTH, RACK_U
    ports = []
    for row in range(2):
        for col in range(12):
            ports.append((72 + col * 32.0, 6 + row * 22.0, 17, 14))
    connections = []
    for row in range(2):
        for col in range(12):
            number = row * 12 + col + 1
            connections.append((f"Port {number}", (72 + col * 32.0 + 8.5) / w, (6 + row * 22.0 + 7.0) / h))
    shapes = [
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#2b3140",
            line="#5e6a80",
            rects=[(0, 0, w, h)],
            properties=[
                ("Manufacturer", "RackCraft"),
                ("Part Number", "RC-PP-24C6"),
                ("Product Description", "1U 24-port CAT6 patch panel, 19 inch"),
                ("Category", "Patch panels"),
                ("Port Count", "24"),
            ],
            connections=connections,
            name="CAT6 patch panel",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#0c0f14",
            line="#74808f",
            line_weight=0.15,
            rects=ports,
            name="RJ45 ports",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=4,
            y=5,
            w=54,
            h=h - 10,
            fill="#181c25",
            line="#303744",
            rects=[(0, 0, 54, h - 10)],
            name="Label field",
        ),
    ]
    return {
        "name": "24-Port CAT6 Patch Panel",
        "width_mm": w,
        "height_mm": h,
        "icon": icon_b64((232, 87, 63), "panel"),
        "shapes": shapes,
    }


def switch_master() -> dict:
    w, h = RACK_WIDTH, RACK_U
    ports = [(61 + col * 16.6, 23, 12, 13) for col in range(24)]
    sfp = [(429, 6, 18, 12), (452, 6, 18, 12)]
    connections = [
        (f"Ethernet {col + 1}", (61 + col * 16.6 + 6) / w, 29.5 / h) for col in range(24)
    ]
    shapes = [
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#1f2530",
            line="#5e6a80",
            rects=[(0, 0, w, h)],
            properties=[
                ("Manufacturer", "RackCraft"),
                ("Part Number", "RC-SW-24G"),
                ("Product Description", "1U 24-port gigabit managed switch, 2 SFP"),
                ("Category", "Network"),
                ("Port Count", "26"),
            ],
            connections=connections,
            name="Gigabit switch",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#0c0f14",
            line="#697588",
            line_weight=0.15,
            rects=ports + sfp,
            name="Switch ports",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=10,
            y=8,
            w=26,
            h=4,
            fill="#e8573f",
            line=None,
            rects=[(0, 0, 26, 4)],
            name="Brand strip",
        ),
    ]
    return {
        "name": "24-Port Gigabit Switch",
        "width_mm": w,
        "height_mm": h,
        "icon": icon_b64((232, 87, 63), "switch"),
        "shapes": shapes,
    }


def cable_manager_master() -> dict:
    w, h = RACK_WIDTH, RACK_U
    fingers = [(26 + i * 37.0, 12, 24, 20) for i in range(12)]
    shapes = [
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#232833",
            line="#5e6a80",
            rects=[(0, 0, w, h)],
            properties=[
                ("Manufacturer", "RackCraft"),
                ("Part Number", "RC-CM-1U"),
                ("Product Description", "1U horizontal cable manager with 12 fingers"),
                ("Category", "Cable management"),
            ],
            name="Cable manager",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#0c0f14",
            line="#303744",
            line_weight=0.2,
            rects=fingers,
            name="Cable fingers",
        ),
    ]
    return {
        "name": "1U Cable Manager",
        "width_mm": w,
        "height_mm": h,
        "icon": icon_b64((232, 87, 63), "cable"),
        "shapes": shapes,
    }


def modular_panel_master() -> dict:
    w, h = RACK_WIDTH, RACK_U
    aperture = w / 24.0
    apertures = [(col * aperture + 2.4, 8, aperture - 4.8, 28) for col in range(24)]
    connections = [(f"Aperture {col + 1}", (col + 0.5) * aperture / w, 0.5) for col in range(24)]
    shapes = [
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#2b3140",
            line="#5e6a80",
            rects=[(0, 0, w, h)],
            properties=[
                ("Manufacturer", "RackCraft"),
                ("Part Number", "RC-MPP-24"),
                ("Product Description", "1U 24-aperture modular patch panel, empty"),
                ("Category", "Modular panels"),
                ("Aperture Count", "24"),
            ],
            connections=connections,
            name="Modular panel",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#0c0f14",
            line="#68748a",
            line_weight=0.15,
            rects=apertures,
            name="Apertures",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=4,
            y=5,
            w=42,
            h=h - 10,
            fill="#181c25",
            line="#303744",
            rects=[(0, 0, 42, h - 10)],
            name="Label field",
        ),
    ]
    return {
        "name": "24-Aperture Modular Patch Panel",
        "width_mm": w,
        "height_mm": h,
        "icon": icon_b64((232, 87, 63), "modular"),
        "shapes": shapes,
    }


def keystone_master() -> dict:
    w, h = 14.7, 17.8
    shapes = [
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#e1e6ee",
            line="#5e6a80",
            line_weight=0.2,
            rects=[(0, 0, w, h)],
            properties=[
                ("Manufacturer", "RackCraft"),
                ("Part Number", "RC-KJ-RJ45"),
                ("Product Description", "CAT6 keystone jack, RJ45, 180°"),
                ("Category", "Keystone jacks"),
                ("Colour", "White"),
            ],
            connections=[("RJ45", 0.5, 0.5)],
            name="Keystone body",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=2.6,
            y=3.0,
            w=w - 5.2,
            h=10.5,
            fill="#0c0f14",
            line="#48505e",
            line_weight=0.15,
            rects=[(0, 0, w - 5.2, 10.5)],
            name="RJ45 opening",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=4.0,
            y=13.0,
            w=6.7,
            h=0.35,
            fill=None,
            line="#d4af5f",
            line_weight=0.35,
            lines=[(0, 0, 6.7, 0)],
            name="Contact",
        ),
    ]
    return {
        "name": "CAT6 Keystone Jack RJ45",
        "width_mm": w,
        "height_mm": h,
        "icon": icon_b64((232, 87, 63), "keystone"),
        "shapes": shapes,
    }


def network_master(
    *,
    name: str,
    part: str,
    description: str,
    accent: str,
    glyph: str,
    icon_kind: str,
) -> dict:
    w, h = 80.0, 50.0
    connections = [("Top", 0.5, 0.0), ("Right", 1.0, 0.5), ("Bottom", 0.5, 1.0), ("Left", 0.0, 0.5)]
    glyph_rects: list[tuple[float, float, float, float]] = []
    glyph_lines: list[tuple[float, float, float, float]] = []
    if glyph == "cloud":
        glyph_rects = [(12, 19, 18, 10), (23, 13, 26, 16), (44, 19, 24, 10), (18, 27, 44, 7)]
    elif glyph == "firewall":
        for row in range(4):
            for col in range(6):
                if (row + col) % 2 == 0:
                    glyph_rects.append((20 + col * 6.8, 14 + row * 5.6, 6.2, 5.0))
    elif glyph == "router":
        glyph_rects = [(30, 17, 20, 12), (35, 20, 4, 3), (41, 20, 4, 3)]
        glyph_lines = [(20, 23, 29, 23), (51, 23, 60, 23), (40, 10, 40, 16), (40, 30, 40, 37)]
    elif glyph == "switch":
        for col in range(8):
            glyph_rects.append((20 + col * 5.2, 20, 4.2, 8.0))
    elif glyph == "server":
        for row in range(3):
            glyph_rects.append((22, 14 + row * 7.0, 36, 5.5))
    elif glyph == "workstation":
        glyph_rects = [(24, 12, 32, 20), (35, 32, 10, 4), (36, 36, 8, 2)]
    else:
        glyph_rects = [(22, 17, 36, 16)]

    shapes = [
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill="#242a34",
            line="#68748a",
            line_weight=0.35,
            rects=[(0, 0, w, h), (3, 3, w - 6, h - 6)],
            properties=[
                ("Manufacturer", "RackCraft"),
                ("Part Number", part),
                ("Product Description", description),
                ("Category", "Network topology"),
            ],
            connections=connections,
            name=f"{name} body",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill=accent,
            line=None,
            rects=[(3, 3, w - 6, 5)],
            name=f"{name} brand",
        ),
        shape_xml(
            page_w=w,
            page_h=h,
            x=0,
            y=0,
            w=w,
            h=h,
            fill=accent if glyph in ("firewall", "router") else "#cfd9e8",
            line=accent if glyph in ("cloud", "switch", "server", "workstation") else None,
            line_weight=0.3,
            rects=glyph_rects,
            lines=glyph_lines,
            name=f"{name} glyph",
        ),
    ]
    return {
        "name": name,
        "width_mm": w,
        "height_mm": h,
        "icon": icon_b64((232, 87, 63), icon_kind),
        "shapes": shapes,
    }


def main() -> None:
    SAMPLES.mkdir(parents=True, exist_ok=True)
    write_vssx(
        SAMPLES / "rackcraft-datacenter.vssx",
        "RackCraft Data Center",
        [rack_master(), patch_panel_master(), switch_master(), cable_manager_master()],
    )
    write_vssx(
        SAMPLES / "rackcraft-modular.vssx",
        "RackCraft Modular",
        [modular_panel_master(), keystone_master()],
    )
    write_vssx(
        SAMPLES / "network-topology.vssx",
        "Network Topology",
        [
            network_master(
                name="Cloud / FAI",
                part="RC-NET-CLOUD",
                description="Internet cloud / ISP uplink symbol",
                accent="#5aa7e8",
                glyph="cloud",
                icon_kind="cloud",
            ),
            network_master(
                name="Firewall",
                part="RC-NET-FW",
                description="Perimeter firewall",
                accent="#e8573f",
                glyph="firewall",
                icon_kind="firewall",
            ),
            network_master(
                name="Router",
                part="RC-NET-RTR",
                description="Edge router",
                accent="#f0b45a",
                glyph="router",
                icon_kind="router",
            ),
            network_master(
                name="Core Switch",
                part="RC-NET-CSW",
                description="Core L3 switch",
                accent="#54d184",
                glyph="switch",
                icon_kind="switch",
            ),
            network_master(
                name="Server",
                part="RC-NET-SRV",
                description="Rack server node",
                accent="#9b8cf0",
                glyph="server",
                icon_kind="server",
            ),
            network_master(
                name="Workstation",
                part="RC-NET-WKS",
                description="End-user workstation",
                accent="#78c8d8",
                glyph="workstation",
                icon_kind="workstation",
            ),
        ],
    )


if __name__ == "__main__":
    main()
