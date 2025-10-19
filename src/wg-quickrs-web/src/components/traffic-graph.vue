<template>
  <div class="w-full h-16 relative overflow-hidden">
    <div id="traffic-graph-box" class="h-16">
      <canvas id="traffic-graph"></canvas>
    </div>

    <!-- Tx/Rx labels on the right -->
    <div
        class="absolute top-2 left-5 flex-col items-end space-y-1 text-sm bg-white/40 px-1 py-1 rounded hidden sm:block">
      <div class="font-semibold whitespace-pre-wrap text-blue-600/30">Tx peak ({{ telem_span }}): {{ tx_peak }}
      </div>
      <div class="font-semibold whitespace-pre-wrap text-green-600/30">Rx peak ({{ telem_span }}): {{ rx_peak }}
      </div>
    </div>

    <!-- Tx/Rx labels on the right -->
    <div
        class="absolute top-2 right-5 flex flex-col items-end space-y-1 text-sm bg-white/40 px-1 py-1 rounded">
      <div class="font-semibold whitespace-pre-wrap text-blue-600/30">Tx: {{ tx_avg }}
      </div>
      <div class="font-semibold whitespace-pre-wrap text-green-600/30">Rx: {{ rx_avg }}
      </div>
    </div>
  </div>
</template>

<script>
const {Chart, LineElement, LinearScale, LineController, PointElement, Filler} = await import("chart.js");
Chart.register(
    LineElement,
    LinearScale,
    LineController,
    PointElement,
    Filler
);

function formatThroughputBits(bytesPerSec) {
  if (!isFinite(bytesPerSec) || bytesPerSec <= 0) return "0 b/s";

  const factor = 1000; // SI scaling
  let value = bytesPerSec * 8.; // convert to bits/s

  const units = ["b/s", "kb/s", "Mb/s", "Gb/s", "Tb/s"];

  let unitIndex = 0;
  while (value >= factor && unitIndex < units.length - 1) {
    value /= factor;
    unitIndex++;
  }

  // round to 1 decimal, drop trailing .0
  const rounded = Math.round(value * 10) / 10;
  const text = rounded % 1 === 0 ? String(rounded) : rounded.toFixed(1);

  return `${text} ${units[unitIndex]}`;
}

export default {
  name: "traffic-graph",
  props: {
    network: {
      type: Object,
      default: {},
    },
    telemetry: {
      type: Object,
      default: {},
    },
  },
  data() {
    return {
      tx_avg: "? b/s",
      rx_avg: "? b/s",
      tx_peak: "? b/s",
      rx_peak: "? b/s",
      telem_span: "?",
      box_div_element: null,
      intervalId: null,
      chart: null,
      prevGraphMax: 0,
    }
  },
  mounted() {
    this.box_div_element = document.getElementById('traffic-graph-box');

    const ctx = document.getElementById('traffic-graph');
    const chartObj = new Chart(ctx, {
      type: "line",
      data: {
        labels: [0],
        datasets: [
          {
            label: "RX",
            borderColor: "rgba(34,197,94,0.2)", // faint green
            backgroundColor: "rgba(34,197,94,0.05)",
            data: [0],
            tension: 0.3,
            fill: "origin",
            pointRadius: 0,
          },
          {
            label: "TX",
            borderColor: "rgba(59,130,246,0.2)", // faint blue
            backgroundColor: "rgba(59,130,246,0.05)",
            data: [0],
            tension: 0.3,
            fill: "origin",
            pointRadius: 0,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: false,
        plugins: {},
        scales: {x: {type: "linear", display: false}, y: {type: "linear", display: false, beginAtZero: true}},
      },
    });
    Object.seal(chartObj);
    this.chart = chartObj;
  },
  watch: {
    telemetry: {
      handler() {
        if (this.telemetry === null) {
          this.resetText();
          return;
        } else if (Object.keys(this.telemetry.data).length < 2) {
          this.resetText();
          return;
        }
        const txs = [];
        const rxs = [];
        const timestamps = [];
        let prev_telem_data = {};

        for (const telem_data of this.telemetry.data) {
          if (Object.keys(prev_telem_data).length === 0) {
            prev_telem_data = telem_data;
            continue;
          }

          let tx = 0,
              rx = 0;
          const ts = (telem_data.timestamp - prev_telem_data.timestamp) / 1000;

          for (const [connection_id, telemetry_details] of Object.entries(telem_data.datum)) {
            if (!prev_telem_data.datum[connection_id]) continue;

            if (connection_id.startsWith(this.network.this_peer)) {
              tx += telemetry_details.transfer_a_to_b - prev_telem_data.datum[connection_id].transfer_a_to_b;
              rx += telemetry_details.transfer_b_to_a - prev_telem_data.datum[connection_id].transfer_b_to_a;
            } else if (connection_id.endsWith(this.network.this_peer)) {
              tx += telemetry_details.transfer_b_to_a - prev_telem_data.datum[connection_id].transfer_b_to_a;
              rx += telemetry_details.transfer_a_to_b - prev_telem_data.datum[connection_id].transfer_a_to_b;
            }
          }

          txs.push(tx / ts);
          rxs.push(rx / ts);
          timestamps.push(telem_data.timestamp);
          prev_telem_data = telem_data;

        }
        const tx_avg = formatThroughputBits(txs.at(-1));
        const rx_avg = formatThroughputBits(rxs.at(-1));
        const maxAvgLen = Math.max(tx_avg.length, rx_avg.length);
        this.tx_avg = tx_avg.padStart(maxAvgLen, " ");
        this.rx_avg = rx_avg.padStart(maxAvgLen, " ");

        let latest_timestamp_dt = timestamps.at(-1) - timestamps.at(-2);
        while (txs.length < this.telemetry.max_len - 1) {
          txs.unshift(0);
          rxs.unshift(0);
          timestamps.unshift(timestamps[0] - latest_timestamp_dt);
        }
        const tx_peak_n = Math.max(...txs);
        const rx_peak_n = Math.max(...rxs);
        const tx_peak = formatThroughputBits(tx_peak_n);
        const rx_peak = formatThroughputBits(rx_peak_n);
        const maxPeakLen = Math.max(tx_peak.length, rx_peak.length);
        this.tx_peak = tx_peak.padStart(maxPeakLen, " ");
        this.rx_peak = rx_peak.padStart(maxPeakLen, " ");
        this.telem_span = `${Math.round((this.telemetry.data.at(-1).timestamp - this.telemetry.data[0].timestamp) / 1000.)}s`;

        // Update the chart directly
        this.chart.data.labels = timestamps;
        this.chart.options.scales.x.min = timestamps[0];
        this.chart.options.scales.x.max = timestamps.at(-1);
        this.chart.data.datasets[0].data = rxs;
        this.chart.data.datasets[1].data = txs;
        let ratio_new_data = latest_timestamp_dt / (timestamps.at(-1) - timestamps[0]);
        this.resetMove(ratio_new_data, Math.max(tx_peak_n, rx_peak_n) * 1.03);
        this.chart.update();
      },
      deep: true
    }
  },
  methods: {
    resetText() {
      this.tx_avg = "? b/s";
      this.rx_avg = "? b/s";
      this.tx_peak = "? b/s";
      this.rx_peak = "? b/s";
      this.telem_span = "?";
    },
    resetMove(ratio, newGraphMax) {
      if (this.intervalId) {
        clearInterval(this.intervalId);
        this.intervalId = null;
      }
      const duration = 1000.;
      const refresh_interval = 40.; // 25Hz
      const total_margin = ratio * this.box_div_element.getBoundingClientRect().width
      this.box_div_element.style.marginLeft = `${Math.round(-2. * total_margin)}px`;
      this.box_div_element.style.transform = `translateX(${Math.round(total_margin)}px)`;

      let start_time = Date.now();
      this.intervalId = setInterval(() => {
        const pos = (Date.now() - start_time) / duration;
        const total_transform = total_margin * (1. - pos);
        this.box_div_element.style.transform = `translateX(${Math.round(total_transform)}px)`;

        // minimize the number of chart.update calls
        if (newGraphMax !== this.prevGraphMax) {
          // If pos > 0.9, complete y-axis scaling animation
          this.chart.options.scales.y.max = this.prevGraphMax + (newGraphMax - this.prevGraphMax) * (pos > 0.9 ? 1. : pos);
          this.prevGraphMax = this.chart.options.scales.y.max;
          this.chart.update();
        }
      }, refresh_interval); // roughly every interval milliseconds
    }
  }
}
</script>
