window.BENCHMARK_DATA = {
  "lastUpdate": 1674262797513,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "94b454ca2f073bd6bfd39041a2b8f42afb726c7b",
          "message": "Merge pull request #723 from ImmanuelSegol/feat/avoid-wasm-error\n\nbug fix - wasm pack compile error",
          "timestamp": "2023-01-20T23:55:57Z",
          "tree_id": "f5b6b2369a4c9dfad0424da9b43a4f22adbad124",
          "url": "https://github.com/zcash/halo2/commit/94b454ca2f073bd6bfd39041a2b8f42afb726c7b"
        },
        "date": 1674262788249,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 102652977,
            "range": "± 9651226",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 5832082,
            "range": "± 687313",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 190192882,
            "range": "± 6822136",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6581673,
            "range": "± 712239",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 256815351,
            "range": "± 7041572",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7686520,
            "range": "± 728773",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 46886,
            "range": "± 543",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164874,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 180103,
            "range": "± 975",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 293505,
            "range": "± 655",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 293495,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 168331,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 183434,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 296750,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 296866,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 352118,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 367158,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 480542,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 480560,
            "range": "± 223",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3696399,
            "range": "± 1818",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6016676,
            "range": "± 3895",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10414736,
            "range": "± 27532",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21865044,
            "range": "± 34728",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41278196,
            "range": "± 37332",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 80960959,
            "range": "± 133275",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7312,
            "range": "± 539",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8483,
            "range": "± 751",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15189,
            "range": "± 668",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18990,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26651,
            "range": "± 3047",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 44537,
            "range": "± 7666",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 99722,
            "range": "± 15221",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 216568,
            "range": "± 40752",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 469278,
            "range": "± 52320",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 868916,
            "range": "± 114678",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1645790,
            "range": "± 176927",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3436810,
            "range": "± 392738",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7994642,
            "range": "± 619667",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15355171,
            "range": "± 1912361",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 33219432,
            "range": "± 2940000",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 72803435,
            "range": "± 988638",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34921,
            "range": "± 943",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35032,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 179555191,
            "range": "± 6239361",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 389708749,
            "range": "± 4135256",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 839849928,
            "range": "± 8869363",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1788022174,
            "range": "± 8370509",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3817447780,
            "range": "± 7695122",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8152365019,
            "range": "± 28662809",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17335283568,
            "range": "± 26095161",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36697890831,
            "range": "± 20987339",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77243040139,
            "range": "± 196268543",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 120176789,
            "range": "± 3266885",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 211144908,
            "range": "± 5586329",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 353368988,
            "range": "± 9390221",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 637542635,
            "range": "± 7883658",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1142399063,
            "range": "± 10097384",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2151035785,
            "range": "± 14442854",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4070883562,
            "range": "± 13157293",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7878359894,
            "range": "± 23738854",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15078930163,
            "range": "± 40830504",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5827714,
            "range": "± 413584",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9106415,
            "range": "± 848094",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14716037,
            "range": "± 980782",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24872380,
            "range": "± 2479694",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42876818,
            "range": "± 2887938",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76420065,
            "range": "± 4243001",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137597956,
            "range": "± 5769311",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 253057365,
            "range": "± 14835595",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 464813356,
            "range": "± 6281645",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}