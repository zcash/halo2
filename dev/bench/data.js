window.BENCHMARK_DATA = {
  "lastUpdate": 1661385522760,
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
          "id": "8fa116aa852c94ef2abcd1447e6ef5cf27ff8372",
          "message": "Merge pull request #642 from zcash/reduce-ast-evaluator-memory-usage\n\nhalo2_proofs: Reduce memory usage of `poly::Evaluator`",
          "timestamp": "2022-08-25T00:00:03+01:00",
          "tree_id": "3a1e59e5d66a3919fb623efc468408ab8f38f2b8",
          "url": "https://github.com/zcash/halo2/commit/8fa116aa852c94ef2abcd1447e6ef5cf27ff8372"
        },
        "date": 1661385516616,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 60414833,
            "range": "± 773395",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2961493,
            "range": "± 31662",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 132472326,
            "range": "± 2639016",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3636289,
            "range": "± 153312",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 183879540,
            "range": "± 1492132",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 3972357,
            "range": "± 62162",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38764,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141552,
            "range": "± 1986",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153719,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 251879,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 251849,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144330,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156684,
            "range": "± 1149",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 254754,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 254717,
            "range": "± 1021",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302288,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314553,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 413158,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 413394,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3014117,
            "range": "± 1442",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5910452,
            "range": "± 3460",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10168841,
            "range": "± 4639",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21182093,
            "range": "± 41898",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 40162695,
            "range": "± 175549",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 81058665,
            "range": "± 614591",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7829,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8770,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16115,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18867,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27731,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 46904,
            "range": "± 950",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 97649,
            "range": "± 7199",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 188351,
            "range": "± 8474",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 385564,
            "range": "± 10770",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 811752,
            "range": "± 23249",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1714596,
            "range": "± 42254",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3694507,
            "range": "± 38731",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8083042,
            "range": "± 230357",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17355512,
            "range": "± 544937",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 37873593,
            "range": "± 408629",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 85928167,
            "range": "± 1116144",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28430,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28526,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 152649752,
            "range": "± 342409",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 327431513,
            "range": "± 696167",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 704960686,
            "range": "± 1629943",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1516460997,
            "range": "± 10207451",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3252431692,
            "range": "± 3982953",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6945578796,
            "range": "± 4534040",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14796091243,
            "range": "± 14691574",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31431763560,
            "range": "± 31099736",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66493325035,
            "range": "± 86659950",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 95814397,
            "range": "± 551377",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 165989815,
            "range": "± 763845",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 293413448,
            "range": "± 1978612",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 536109668,
            "range": "± 7458230",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 994629970,
            "range": "± 813131",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1886655875,
            "range": "± 140145273",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3592453187,
            "range": "± 8085865",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6950119031,
            "range": "± 6304588",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13382702684,
            "range": "± 27942563",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5151537,
            "range": "± 14138",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7921932,
            "range": "± 176097",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12803678,
            "range": "± 315597",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21491751,
            "range": "± 193984",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37315045,
            "range": "± 292446",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66577282,
            "range": "± 546607",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120135008,
            "range": "± 1545120",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 221930695,
            "range": "± 9876173",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 402953611,
            "range": "± 1707107",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}