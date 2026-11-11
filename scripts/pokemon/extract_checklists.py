"""Produce review candidates from local publisher PDFs. Requires Python 3 + Poppler.

Usage: python3 scripts/pokemon/extract_checklists.py plan.json pdf-directory output.json
PDF filenames must be <contentSha256>.pdf. This offline tool never publishes data.
Only the consecutive main numbered section is extracted. Symbols need visual review.
"""
import hashlib
import json
import pathlib
import re
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET


def extract(pdf, count):
    with tempfile.TemporaryDirectory() as directory:
        xml = pathlib.Path(directory) / "words.xml"
        subprocess.run(["pdftotext", "-bbox-layout", str(pdf), str(xml)], check=True, timeout=30)
        words = []
        for word in ET.parse(xml).getroot().iter():
            if word.tag.endswith("}word"):
                words.append({"text": word.text or "", "x": float(word.attrib["xMin"]),
                              "y": (float(word.attrib["yMin"]) + float(word.attrib["yMax"])) / 2,
                              "h": float(word.attrib["yMax"]) - float(word.attrib["yMin"])})
    numbers = [w for w in words if re.fullmatch(r"\d{3}", w["text"]) and any(
        t["text"] == "■" and 0 < t["x"] - w["x"] < 20 and abs(t["y"] - w["y"]) < 3
        for t in words)]
    anchors = []
    for word in sorted(numbers, key=lambda w: w["x"]):
        if not anchors or word["x"] - anchors[-1] > 8:
            anchors.append(word["x"])
        word["col"] = anchors[-1]
    numbers.sort(key=lambda w: (w["col"], w["y"]))
    numbers = numbers[:count]
    if [int(w["text"]) for w in numbers] != list(range(1, count + 1)):
        raise ValueError("Missing/duplicate/nonconsecutive numbers: review the PDF layout")
    columns = sorted({w["col"] for w in numbers})
    cards = []
    for number in numbers:
        column = [w for w in numbers if w["col"] == number["col"]]
        right = min([x for x in columns if x > number["x"] + 5] + [10000])
        chosen = [w for w in words if number["x"] + 12 < w["x"] < right - 5
                  and w["text"] != "■" and abs(w["y"] - number["y"]) <= 5
                  and abs(w["h"] - number["h"]) < 1
                  and min(column, key=lambda t: abs(t["y"] - w["y"])) is number]
        chosen.sort(key=lambda w: (round(w["y"] / 2), w["x"]))
        name = " ".join(w["text"] for w in chosen)
        if not name or len(name) > 200:
            raise ValueError("Unclear name: review the PDF")
        cards.append({"collectorNumber": number["text"], "name": name})
    return cards


def main():
    plan = json.loads(pathlib.Path(sys.argv[1]).read_text())
    folder = pathlib.Path(sys.argv[2])
    if not 1 <= len(plan["sets"]) <= 24:
        raise ValueError("Select 1–24 checklists")
    for checklist in plan["sets"]:
        digest = checklist["contentSha256"]
        if not re.fullmatch(r"[a-f0-9]{64}", digest) or not 1 <= checklist["expectedCount"] <= 500:
            raise ValueError("Invalid digest or count")
        pdf = folder / (digest + ".pdf")
        if pdf.stat().st_size > 10 * 1024 * 1024:
            raise ValueError("PDF exceeds 10 MiB")
        if hashlib.sha256(pdf.read_bytes()).hexdigest() != digest:
            raise ValueError("PDF checksum changed; review the source again")
        checklist["cards"] = extract(pdf, checklist["expectedCount"])
    plan["reviewedBy"] = None
    with open(sys.argv[3], "x", encoding="utf-8") as output:
        json.dump(plan, output, ensure_ascii=False, indent=2)
        output.write("\n")


if __name__ == "__main__":
    main()
