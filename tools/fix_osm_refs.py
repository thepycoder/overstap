"""Drop ways that reference nodes missing from the extract (and relations
pointing to dropped ways), so MOTIS/osr doesn't hit osmium::invalid_location."""
import sys
import osmium
from osmium.index import IdSet

src, dst = sys.argv[1], sys.argv[2]

nodes = IdSet()
class N(osmium.SimpleHandler):
    def node(self, n): nodes.set(n.id)
N().apply_file(src)

dropped_ways = IdSet()
stats = {"ways_dropped": 0, "rels_dropped": 0}
writer = osmium.SimpleWriter(dst)

class W(osmium.SimpleHandler):
    def node(self, n): writer.add_node(n)
    def way(self, w):
        if all(nodes.get(r.ref) for r in w.nodes):
            writer.add_way(w)
        else:
            dropped_ways.set(w.id); stats["ways_dropped"] += 1
    def relation(self, r):
        if any(m.type == "w" and dropped_ways.get(m.ref) for m in r.members):
            stats["rels_dropped"] += 1
            return
        writer.add_relation(r)
W().apply_file(src)
writer.close()
print(stats)
