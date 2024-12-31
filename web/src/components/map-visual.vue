<template>

  <div id="graph"></div>

</template>

<script>
import ForceGraph from "force-graph";
import FastEqual from "fast-deep-equal";
import WireGuardHelper from "../js/wg-helper.js";


export default {
  name: "map-visual",
  props: {
    network: {
      type: Object,
      default: {},
    },
  },
  data() {
    return {
      initializedGraph: false,
      graph: null,
    }
  },
  watch: {
    network: function (newVal, oldVal) { // watch it
      if (FastEqual(newVal, oldVal)) {
        return;
      }

      if (this.initializedGraph) {
        try {
          this.graph.graphData(this.calculateForceGraphData(newVal));
        } catch (e) {
          console.log(e);
        }
      }

      if (!this.initializedGraph) {
        try {
          this.graph = ForceGraph()(document.getElementById('graph'))
              // .nodeCanvasObject((node, ctx) => {
              //   if (this.peerAvatarCanvases[node.id]) {
              //     ctx.drawImage(this.peerAvatarCanvases[node.id], node.x - node.size / 2, node.y - node.size / 2, node.size, node.size);
              //   } else {
              //     const img = new Image();
              //     img.src = node.mobility === 'static' ? staticPeerIconSrc : roamingPeerIconSrc;
              //     ctx.drawImage(this.getGraphNodeIcon(img), node.x - node.size / 2, node.y - node.size / 2, node.size, node.size);
              //   }
              // })
              .nodePointerAreaPaint((node, color, ctx) => {
                ctx.beginPath();
                ctx.arc(node.x, node.y, node.size / 2, 0, Math.PI * 2, true);
                ctx.fillStyle = color;
                ctx.fill();
              })
              .height(document.getElementById('graph').clientHeight)
              .width(document.getElementById('graph').clientWidth)
              .d3Force('center', null)
              .zoomToFit(100, 20)
              .nodeId('id')
              .nodeLabel('name')
              .nodeColor('color')
              .linkSource('source')
              .linkTarget('target')
              .linkAutoColorBy('color')
              .linkDirectionalParticles('particleCount')
              .linkWidth('strength')
              .cooldownTicks(10);

          this.graph.onEngineStop(() => this.graph.zoomToFit(400, 20));
          this.graph.onBackgroundClick(() => this.graph.zoomToFit(400, 20));
          this.graph.onNodeClick(node => {
            // Center/zoom on node
            this.graph.centerAt(node.x, node.y, 400);
            this.graph.zoom(8, 400);

            // this.peerEditWindow.id = node.id;
          });

          this.graph.graphData(this.calculateForceGraphData(newVal));
          this.initializedGraph = true;
        } catch (e) {
          console.log(e);
        }
      }
    }
  },
  mounted: function () {

  },
  computed: {},
  methods: {
    calculateForceGraphData(network) {
      const peerSize = {};
      Object.keys(network.peers).forEach(peerId => {
        peerSize[peerId] = 1;
      });
      const forceG = {nodes: [], links: []};
      for (const [connectionId, connectionDetails] of Object.entries(network.connections)) {
        if (connectionDetails.enabled) {
          const {a, b} = WireGuardHelper.getConnectionPeers(connectionId);
          const linkColorStrength = 1
              + network.staticPeerIds.includes(a)
              + network.staticPeerIds.includes(b);
          let color = '';
          // eslint-disable-next-line default-case
          switch (linkColorStrength) {
            case 1:
              color = 'rgb(229 231 235)';
              break;
            case 2:
              color = 'rgb(209 213 219)';
              break;
            case 3:
              color = 'rgb(107 114 128)';
              break;
          }
          forceG.links.push({
            source: a, target: b, particleCount: 0, color, strength: linkColorStrength,
          });
          forceG.links.push({
            source: b, target: a, particleCount: 0, color, strength: linkColorStrength,
          });
          for (const ab of [a, b]) {
            peerSize[ab] += network.staticPeerIds.includes(ab) ? 0.925 : 0.0625;
            peerSize[ab] += connectionDetails.enabled ? 0.125 : 0.03125;
          }
        }
      }

      for (const [peerId, peerDetails] of Object.entries(network.peers)) {
        forceG.nodes.push({
          id: peerId,
          name: peerDetails.name,
          mobility: peerDetails.mobility,
          size: Math.sqrt(peerSize[peerId]) * 7,
          color: network.staticPeerIds.includes(peerId) ? 'rgb(21 128 61)' : 'rgb(7 89 133)',
          // icon: this.peerAvatarCanvases[peerId],
        });
      }
      return forceG;
    }
  }
}
</script>

<style scoped>

</style>