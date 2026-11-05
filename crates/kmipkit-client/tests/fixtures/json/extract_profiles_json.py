"""Extract the literal deterministic JSON examples from the pinned Profiles HTML.

This is an offline fixture maintenance utility, never part of library builds.
Only HTML markup/entities and indentation NBSPs are removed; JSON token text,
versions, tags, values, and timestamps are retained exactly.
"""
from html.parser import HTMLParser
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[5]
SOURCE = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"

class Text(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.parts = []
    def handle_data(self, data):
        self.parts.append(data)
    def handle_endtag(self, tag):
        if tag == "p":
            self.parts.append("\n")

source = SOURCE.read_bytes().decode("cp1252")
section = source.index('name="_Toc32324439"')
start = source.index("The normative corresponding wire encoding in JSON", section)
end = source.index("<h2", start)
parser = Text()
parser.feed(source[start:end])
plain = "".join(parser.parts).replace("\xa0", " ")
decoder = json.JSONDecoder()
position = 0
for filename in ["request-time-0.json", "response-time-0.json", "request-time-1.json"]:
    position = plain.index("{", position)
    _, length = decoder.raw_decode(plain[position:])
    Path(__file__).with_name(filename).write_bytes((plain[position:position+length] + "\n").encode("utf-8"))
    position += length
