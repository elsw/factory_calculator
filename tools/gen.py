#!/usr/bin/env python3
"""Generate Satisfactory Plus recipe TOML files from the k-mods catalog export."""
import json, re, sys, os
from fractions import Fraction
from collections import defaultdict, Counter

SCRATCH = os.path.dirname(os.path.abspath(__file__))
def load(name): return json.load(open(os.path.join(SCRATCH, name)))

recipes_doc = load('seg_recipes.json')
R = recipes_doc['recipes']
DATA_VERSION = recipes_doc.get('dataVersion', '?')
P = {p['id']: p for p in load('seg_parts.json')['parts']}
M = {m['id']: m for m in load('seg_machines.json')['machines']}

def part_name(pid): return P.get(pid, {}).get('name', pid)
def mach_name(mid): return M.get(mid, {}).get('name', mid)

def fmt_rate(v):
    """Format a perMinute value (int, decimal, or fraction string) as a TOML number token."""
    if v is None or v == 'None':
        return None
    f = Fraction(str(v))
    if f.denominator == 1:
        return str(f.numerator)
    val = float(f)
    s = f'{val:.6f}'.rstrip('0').rstrip('.')
    return s

def rate_of(port):
    r = fmt_rate(port.get('perMinute'))
    if r is None:               # Space Elevator phases carry amount, not perMinute
        r = fmt_rate(port.get('amount'))
    return r if r is not None else '0'

prod = [r for r in R if r.get('producerIds')]

# ---- assign each recipe to its primary (first) producer; collect all producers ----
by_primary = defaultdict(list)
for r in prod:
    primary = mach_name(r['producerIds'][0])
    by_primary[primary].append(r)

def slug(name):
    s = name.lower()
    s = s.replace('.', ' ').replace('!', ' ').replace('/', ' ')
    s = re.sub(r'[^a-z0-9]+', '_', s).strip('_')
    s = re.sub(r'_+', '_', s)
    return s

def toml_key(name):
    # Always quote table keys: valid for any name (spaces, punctuation).
    return '["' + name.replace('\\', '\\\\').replace('"', '\\"') + '"]'

def toml_str(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'

def arr_str(items):
    return '[' + ', '.join(toml_str(i) for i in items) + ']'

def arr_num(items):
    return '[' + ', '.join(items) + ']'

def collapse_ports(ports):
    """Sum rates for repeated part ids, preserve first-seen order."""
    order = []
    agg = {}
    for p in ports:
        pid = p['partId']
        r = Fraction(rate_of(p))
        if pid in agg:
            agg[pid] += r
        else:
            agg[pid] = r
            order.append(pid)
    out = []
    for pid in order:
        f = agg[pid]
        if f.denominator == 1:
            tok = str(f.numerator)
        else:
            tok = f'{float(f):.6f}'.rstrip('0').rstrip('.')
        out.append((part_name(pid), tok))
    return out

os.makedirs(os.path.join(SCRATCH, 'out'), exist_ok=True)
summary = []

for machine, rlist in sorted(by_primary.items()):
    lines = []
    lines.append(f'# Satisfactory Plus recipes produced in the {machine}.')
    lines.append(f'# Auto-generated from the k-mods.com catalog export (dataVersion {DATA_VERSION}).')
    lines.append('# "produced_in" lists every machine that can run the recipe; the')
    lines.append('# [default] applies to all entries unless an entry overrides it.')
    lines.append('')
    lines.append('[default]')
    lines.append(f'produced_in = {arr_str([machine])}')
    lines.append('')

    used_keys = Counter()
    for r in rlist:
        inputs = [p for p in r['ports'] if p['direction'] == 'input']
        outputs = [p for p in r['ports'] if p['direction'] == 'output']
        in_pairs = collapse_ports(inputs)
        out_pairs = collapse_ports(outputs)

        # unique, disambiguated table key
        base = r['name']
        used_keys[base] += 1
        key = base if used_keys[base] == 1 else f'{base} ({used_keys[base]})'

        # Power generators and material sinks consume inputs but emit no part.
        primary_name = out_pairs[0][0] if out_pairs else None
        byproducts = out_pairs[1:]

        producers = [mach_name(m) for m in r['producerIds']]
        # dedupe preserving order
        seen = set(); producers = [m for m in producers if not (m in seen or seen.add(m))]

        lines.append(toml_key(key))
        if in_pairs:
            lines.append(f'inputs = {arr_str([n for n, _ in in_pairs])}')
            lines.append(f'input_rates = {arr_num([t for _, t in in_pairs])}')
        else:
            lines.append('inputs = []')
            lines.append('input_rates = []')
        # output field only when the table key is not the output item name
        if primary_name is not None and key != primary_name:
            lines.append(f'output = {toml_str(primary_name)}')
        out_rates = ([out_pairs[0][1]] + [t for _, t in byproducts]) if out_pairs else []
        lines.append(f'output_rates = {arr_num(out_rates)}')
        if byproducts:
            lines.append(f'byproducts = {arr_str([n for n, _ in byproducts])}')
        if producers != [machine]:
            lines.append(f'produced_in = {arr_str(producers)}')
        lines.append('')

    fname = slug(machine) + '.toml'
    with open(os.path.join(SCRATCH, 'out', fname), 'w') as f:
        f.write('\n'.join(lines).rstrip() + '\n')
    summary.append((fname, machine, len(rlist)))

print(f'Wrote {len(summary)} files, {sum(s[2] for s in summary)} recipes total.')
for fn, mc, n in summary:
    print(f'  {fn:40s} {n:4d}  ({mc})')
