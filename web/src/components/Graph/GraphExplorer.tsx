import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import * as d3 from "d3";
import type { GraphData, GraphNode } from "../../api/types";

interface SimNode extends GraphNode, d3.SimulationNodeDatum {}
interface SimLink extends d3.SimulationLinkDatum<SimNode> {
  relation_type: string;
  confidence: number;
}

const COLORS: Record<string, string> = {
  document: "#64ffda",
  entity: "#c084fc",
};

const EDGE_COLORS: Record<string, string> = {
  references: "#64748b",
  follows_up: "#fbbf24",
  renews: "#34d399",
  responds_to: "#60a5fa",
  invoices_for: "#f87171",
};

export default function GraphExplorer({
  data,
  focusId,
}: {
  data: GraphData;
  focusId?: string;
}) {
  const svgRef = useRef<SVGSVGElement>(null);
  const navigate = useNavigate();

  useEffect(() => {
    if (!svgRef.current || data.nodes.length === 0) return;

    const svg = d3.select(svgRef.current);
    const { width, height } = svgRef.current.getBoundingClientRect();

    svg.selectAll("*").remove();

    const g = svg.append("g");

    // Zoom
    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.2, 4])
      .on("zoom", (e) => g.attr("transform", e.transform));
    svg.call(zoom);

    const nodes: SimNode[] = data.nodes.map((n) => ({ ...n }));
    const links: SimLink[] = data.links.map((l) => ({
      source: l.source,
      target: l.target,
      relation_type: l.relation_type,
      confidence: l.confidence,
    }));

    const sim = d3
      .forceSimulation(nodes)
      .force(
        "link",
        d3
          .forceLink<SimNode, SimLink>(links)
          .id((d) => d.id)
          .distance(120),
      )
      .force("charge", d3.forceManyBody().strength(-300))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collision", d3.forceCollide(40));

    // Links
    const link = g
      .append("g")
      .selectAll("line")
      .data(links)
      .join("line")
      .attr("stroke", (d) => EDGE_COLORS[d.relation_type] ?? "#475569")
      .attr("stroke-width", (d) => 1 + d.confidence)
      .attr("stroke-opacity", 0.6);

    // Link labels
    const linkLabel = g
      .append("g")
      .selectAll("text")
      .data(links)
      .join("text")
      .text((d) => d.relation_type.replace(/_/g, " "))
      .attr("font-size", 9)
      .attr("fill", "#64748b")
      .attr("text-anchor", "middle");

    // Nodes
    const node = g
      .append("g")
      .selectAll<SVGGElement, SimNode>("g")
      .data(nodes)
      .join("g")
      .attr("cursor", "pointer")
      .call(
        d3
          .drag<SVGGElement, SimNode>()
          .on("start", (event, d) => {
            if (!event.active) sim.alphaTarget(0.3).restart();
            d.fx = d.x;
            d.fy = d.y;
          })
          .on("drag", (event, d) => {
            d.fx = event.x;
            d.fy = event.y;
          })
          .on("end", (event, d) => {
            if (!event.active) sim.alphaTarget(0);
            d.fx = null;
            d.fy = null;
          }),
      );

    node
      .append("circle")
      .attr("r", (d) => (d.id === focusId ? 20 : d.type === "document" ? 14 : 10))
      .attr("fill", (d) => COLORS[d.type] ?? "#64ffda")
      .attr("fill-opacity", (d) => (d.id === focusId ? 0.3 : 0.15))
      .attr("stroke", (d) => COLORS[d.type] ?? "#64ffda")
      .attr("stroke-width", (d) => (d.id === focusId ? 2.5 : 1.5));

    node
      .append("text")
      .text((d) => {
        const label = d.label ?? d.id;
        return label.length > 24 ? label.slice(0, 22) + "..." : label;
      })
      .attr("dy", (d) => (d.type === "document" ? 26 : 20))
      .attr("text-anchor", "middle")
      .attr("font-size", 10)
      .attr("fill", "#94a3b8");

    node.on("click", (_, d) => {
      if (d.type === "document") navigate(`/documents/${d.id}`);
    });

    sim.on("tick", () => {
      link
        .attr("x1", (d) => (d.source as SimNode).x!)
        .attr("y1", (d) => (d.source as SimNode).y!)
        .attr("x2", (d) => (d.target as SimNode).x!)
        .attr("y2", (d) => (d.target as SimNode).y!);

      linkLabel
        .attr(
          "x",
          (d) =>
            ((d.source as SimNode).x! + (d.target as SimNode).x!) / 2,
        )
        .attr(
          "y",
          (d) =>
            ((d.source as SimNode).y! + (d.target as SimNode).y!) / 2 - 6,
        );

      node.attr("transform", (d) => `translate(${d.x},${d.y})`);
    });

    return () => {
      sim.stop();
    };
  }, [data, focusId, navigate]);

  return (
    <svg
      ref={svgRef}
      className="h-full w-full"
      style={{ minHeight: 400 }}
    />
  );
}
