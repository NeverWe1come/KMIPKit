"""Linear parser for generated Rust route-call match arms."""


def parse_route_arm(line):
    """Return the symbol and body from a generated match arm, if well-formed."""
    if not line.startswith('"') or not line.endswith(','):
        return None
    separator = line.find('" => ', 1)
    if separator < 0:
        return None
    return line[1:separator], line[separator + 5:-1]
