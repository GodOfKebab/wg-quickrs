<template>

  <div id="graph"></div>

</template>

<script>
import ForceGraph from "force-graph";

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
      // console.log('Prop changed: ', newVal, ' | was: ', oldVal)
      if (this.initializedGraph) {
        try {
          this.graph.graphData(this.forceGraphData);
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
              .nodeAutoColorBy('mobility')
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

          this.graph.graphData(this.forceGraphData);
          this.initializedGraph = true;
        } catch (e) {
          console.log(e);
        }
      }
    }
  },
  mounted: function () {
  },
  computed: {
    forceGraphData() {
      const peerSize = {};
      Object.keys(this.network.peers).forEach(peerId => {
        peerSize[peerId] = 1;
      });
      const forceG = {nodes: [], links: []};
      // for (const [connectionId, connectionDetails] of Object.entries(this.network.connections)) {
      //   if (connectionDetails.enabled) {
      //     const {a, b} = WireGuardHelper.getConnectionPeers(connectionId);
      //     const linkColorStrength = 1
      //         + Object.keys(this.staticPeers).includes(a)
      //         + Object.keys(this.staticPeers).includes(b);
      //     let color = '';
      //     // eslint-disable-next-line default-case
      //     switch (linkColorStrength) {
      //       case 1:
      //         color = 'rgb(229 231 235)';
      //         break;
      //       case 2:
      //         color = 'rgb(209 213 219)';
      //         break;
      //       case 3:
      //         color = 'rgb(107 114 128)';
      //         break;
      //     }
      //     forceG.links.push({
      //       source: a, target: b, particleCount: 0, color, strength: linkColorStrength,
      //     });
      //     forceG.links.push({
      //       source: b, target: a, particleCount: 0, color, strength: linkColorStrength,
      //     });
      //     for (const ab of [a, b]) {
      //       peerSize[ab] += Object.keys(this.staticPeers).includes(ab) ? 0.25 : 0.0625;
      //       peerSize[ab] += connectionDetails.enabled ? 0.125 : 0.03125;
      //     }
      //   }
      // }

      for (const [peerId, peerDetails] of Object.entries(this.network.peers)) {
        forceG.nodes.push({
          id: peerId,
          name: peerDetails.name,
          mobility: peerDetails.mobility,
          size: Math.sqrt(peerSize[peerId]) * 7,
          // icon: this.peerAvatarCanvases[peerId],
        });
      }
      return forceG;
    },
  },
  methods: {}
}
</script>

<style scoped>

</style>