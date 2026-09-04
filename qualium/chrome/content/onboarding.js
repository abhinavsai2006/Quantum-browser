// Qualium Onboarding Controller — SRS Section 49 Flow
const Onboarding = {
  startConnecting() {
    const s1 = document.getElementById("step-1");
    const sc = document.getElementById("step-connecting");
    if (s1 && sc) {
      s1.classList.remove("active");
      s1.style.display = "none";
      sc.classList.add("active");
      sc.style.display = "block";
    }

    setTimeout(() => {
      const net = document.getElementById("step-net");
      if (net) {
        net.style.color = "#f1f5f9";
        const dot = net.querySelector(".step-dot");
        if (dot) {
          dot.style.background = "#34d399";
          dot.style.boxShadow = "0 0 8px rgba(52, 211, 153, 0.4)";
        }
      }
    }, 600);

    setTimeout(() => {
      const pqc = document.getElementById("step-pqc");
      if (pqc) {
        pqc.style.color = "#f1f5f9";
        const dot = pqc.querySelector(".step-dot");
        if (dot) {
          dot.style.background = "#34d399";
          dot.style.boxShadow = "0 0 8px rgba(52, 211, 153, 0.4)";
        }
      }
    }, 1200);

    setTimeout(() => {
      const circuit = document.getElementById("step-circuit");
      if (circuit) {
        circuit.style.color = "#f1f5f9";
        const dot = circuit.querySelector(".step-dot");
        if (dot) {
          dot.style.background = "#34d399";
          dot.style.boxShadow = "0 0 8px rgba(52, 211, 153, 0.4)";
        }
      }
      const res = document.getElementById("conn-result");
      if (res) res.style.display = "block";
    }, 1800);

    setTimeout(() => {
      window.location.href = "newtab.xhtml";
    }, 2500);
  }
};
