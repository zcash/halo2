window.BENCHMARK_DATA = {
  "lastUpdate": 1665235870275,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ccfe3e8ccd63ce522863c83dc71f42e497db921a",
          "message": "Merge pull request #671 from zcash/refactor-lookup\n\n`lookup::prover`: Compress expressions and cosets in `commit_permuted`",
          "timestamp": "2022-10-08T13:29:44+01:00",
          "tree_id": "f1c04996ef1320bb4e7ba187c76355e7d2c40702",
          "url": "https://github.com/zcash/halo2/commit/ccfe3e8ccd63ce522863c83dc71f42e497db921a"
        },
        "date": 1665235863341,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 63221026,
            "range": "± 8553380",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3273272,
            "range": "± 196079",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 144048175,
            "range": "± 11540116",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4122569,
            "range": "± 196958",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 193733156,
            "range": "± 9087586",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4317173,
            "range": "± 252914",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 40053,
            "range": "± 2527",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 152310,
            "range": "± 11772",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 162890,
            "range": "± 11280",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 281839,
            "range": "± 19286",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 240250,
            "range": "± 15780",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 141805,
            "range": "± 9269",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 154554,
            "range": "± 9456",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 281956,
            "range": "± 25910",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 249488,
            "range": "± 19994",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 309919,
            "range": "± 23080",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 318831,
            "range": "± 17241",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 415155,
            "range": "± 31929",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 408189,
            "range": "± 39590",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3015012,
            "range": "± 225175",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6270598,
            "range": "± 188584",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10611970,
            "range": "± 246244",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22659616,
            "range": "± 622838",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43590940,
            "range": "± 1505100",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 92026005,
            "range": "± 3029742",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8116,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8951,
            "range": "± 871",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 17884,
            "range": "± 1369",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 22890,
            "range": "± 1522",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 31924,
            "range": "± 2233",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 54584,
            "range": "± 5545",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 103555,
            "range": "± 11943",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 199306,
            "range": "± 15488",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 405993,
            "range": "± 35413",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 850963,
            "range": "± 49795",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1973123,
            "range": "± 144370",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4520327,
            "range": "± 369370",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9271370,
            "range": "± 745753",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 19724127,
            "range": "± 1588722",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 41877209,
            "range": "± 3076471",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 94598082,
            "range": "± 7346029",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28459,
            "range": "± 1385",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28110,
            "range": "± 1993",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 163425988,
            "range": "± 5779326",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 338294660,
            "range": "± 10801747",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 757007008,
            "range": "± 19625325",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1578434135,
            "range": "± 41602160",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3499083960,
            "range": "± 95115551",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 7705088717,
            "range": "± 439205225",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 15635052694,
            "range": "± 197351902",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 33211315183,
            "range": "± 852848599",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 74645500470,
            "range": "± 2248552558",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 109103332,
            "range": "± 5060607",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 179513925,
            "range": "± 3430793",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 313912306,
            "range": "± 13260549",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 614360152,
            "range": "± 26289587",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1105132688,
            "range": "± 27006839",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2113772375,
            "range": "± 43967909",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3834502570,
            "range": "± 111200053",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7481697076,
            "range": "± 171283969",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 14356188768,
            "range": "± 204045403",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5293401,
            "range": "± 335796",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8514147,
            "range": "± 548142",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14117059,
            "range": "± 908705",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 23451443,
            "range": "± 1359974",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 39291395,
            "range": "± 2150417",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 72666451,
            "range": "± 4245515",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 129226623,
            "range": "± 7550559",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 233333935,
            "range": "± 13442083",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 437520376,
            "range": "± 27403363",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}