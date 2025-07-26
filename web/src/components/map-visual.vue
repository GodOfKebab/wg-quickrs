<template>

  <div id="graph"></div>

</template>

<script>
import ForceGraph from "force-graph";
import FastEqual from "fast-deep-equal";
import WireGuardHelper from "../js/wg-helper.js";

import staticNodeIcon from "../icons/svgrepo/globe-05-svgrepo-com.svg";
import roamingNodeIcon from "../icons/svgrepo/rss-02-svgrepo-com.svg";
import thisNodeMarker from "../icons/flowbite/home.svg";

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
              .nodeCanvasObject((node, ctx) => {
                const img = new Image();
                img.src = node.endpoint.enabled ? staticNodeIcon : roamingNodeIcon;
                const cis = this.getGraphNodeIcon(img, 500);
                ctx.drawImage(cis, node.x - node.size / 2, node.y - node.size / 2, node.size, node.size);
                if (node.id === this.network.this_peer) {
                  const marker_img = new Image();
                  marker_img.src = thisNodeMarker;
                  const marker = this.getGraphNodeIcon(marker_img, 500);
                  ctx.drawImage(marker, node.x, node.y, node.size / 2, node.size / 2);
                }
              })
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
              // .nodeColor('color')
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

            // this.peer_selected = node.id;
            // console.log(node.id)
            this.$emit('peer-selected', node.id);
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
              + network.static_peer_ids.includes(a)
              + network.static_peer_ids.includes(b);
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
          // for (const ab of [a, b]) {
          //   peerSize[ab] += network.static_peer_ids.includes(ab) ? 0.925 : 0.0625;
          //   peerSize[ab] += connectionDetails.enabled ? 0.125 : 0.03125;
          // }
        }
      }

      for (const [peerId, peerDetails] of Object.entries(network.peers)) {
        forceG.nodes.push({
          id: peerId,
          name: peerDetails.name,
          endpoint: peerDetails.endpoint,
          size: Math.sqrt(peerSize[peerId]) * 7,
          color: network.static_peer_ids.includes(peerId) ? 'rgb(21 128 61)' : 'rgb(7 89 133)',
          // icon: this.peerAvatarCanvases[peerId],
        });
      }
      return forceG;
    },
    getGraphNodeIcon(image, size) {
      const tmpCanvas = document.createElement('canvas');
      const tmpCtx = tmpCanvas.getContext('2d');

      tmpCanvas.width = size;
      tmpCanvas.height = size;

      // draw the cached images to temporary canvas and return the context
      tmpCtx.save();
      tmpCtx.beginPath();
      tmpCtx.arc(size / 2, size / 2, size / 2, 0, Math.PI * 2, true);
      tmpCtx.closePath();
      tmpCtx.clip();
      tmpCtx.fillStyle = 'rgb(249 250 251)';
      tmpCtx.fillRect(0, 0, size, size);
      tmpCtx.drawImage(image, size / 4, size / 4, size / 2, size / 2);
      return tmpCanvas;
    },
  }
}
</script>

<style scoped>

</style>