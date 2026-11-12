window.BENCHMARK_DATA = {
  "lastUpdate": 1794501126591,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "be.danny.willems@gmail.com",
            "name": "Danny Willems",
            "username": "dannywillems"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "50715ff9880ba3310048c1432c942cc2bb9b36a1",
          "message": "Merge pull request #983 from upbqdn/upstream/poseidon-lib-bench\n\nDisable libtest benches for the halo2_poseidon library",
          "timestamp": "2026-11-12T16:29:12+01:00",
          "tree_id": "a1b3460cf31a1d2e9cfa21f1d1a8594636345347",
          "url": "https://github.com/zcash/halo2/commit/50715ff9880ba3310048c1432c942cc2bb9b36a1"
        },
        "date": 1794501119035,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 63890056,
            "range": "± 2893831",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2585980,
            "range": "± 30450",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 119092853,
            "range": "± 3079864",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3000995,
            "range": "± 30360",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 159890586,
            "range": "± 5906327",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 3230533,
            "range": "± 42295",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 30026,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 105765,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 114288,
            "range": "± 1160",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 192497,
            "range": "± 514",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 192582,
            "range": "± 2276",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 107871,
            "range": "± 1364",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 116215,
            "range": "± 4361",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 194695,
            "range": "± 1299",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 194731,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 225699,
            "range": "± 1306",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 234430,
            "range": "± 935",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 312504,
            "range": "± 2742",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 312523,
            "range": "± 2666",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2401220,
            "range": "± 6591",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 3728631,
            "range": "± 18129",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 6826112,
            "range": "± 70551",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 17242563,
            "range": "± 184278",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 33835433,
            "range": "± 184588",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 66642319,
            "range": "± 236924",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8230,
            "range": "± 1187",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8762,
            "range": "± 1171",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 11840,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 15917,
            "range": "± 1368",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 24734,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 39048,
            "range": "± 1052",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 82467,
            "range": "± 2561",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 152622,
            "range": "± 2605",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 283423,
            "range": "± 3301",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 550486,
            "range": "± 9615",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1115762,
            "range": "± 17905",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 2308218,
            "range": "± 40649",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 4847813,
            "range": "± 85870",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 10285463,
            "range": "± 178967",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 21880565,
            "range": "± 496932",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 47439879,
            "range": "± 2245330",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 23800,
            "range": "± 373",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 23946,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/8",
            "value": 2542940,
            "range": "± 27651",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/9",
            "value": 3922952,
            "range": "± 269623",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/10",
            "value": 6952173,
            "range": "± 40106",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/11",
            "value": 11534857,
            "range": "± 452982",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/12",
            "value": 21052750,
            "range": "± 879725",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/13",
            "value": 37257861,
            "range": "± 1482058",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/14",
            "value": 71332588,
            "range": "± 1260965",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/15",
            "value": 127240639,
            "range": "± 808717",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 114056130,
            "range": "± 2810795",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 239149460,
            "range": "± 18625414",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 493031359,
            "range": "± 11322768",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1052054951,
            "range": "± 22403954",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 2283815022,
            "range": "± 30939613",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 4875677563,
            "range": "± 115122273",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 10121113051,
            "range": "± 299226676",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 21933560476,
            "range": "± 296700080",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 45755365936,
            "range": "± 601315396",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 67091443,
            "range": "± 380435",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 105920429,
            "range": "± 464363",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 182442388,
            "range": "± 606844",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 318743829,
            "range": "± 979879",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 588417171,
            "range": "± 4102399",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1106009080,
            "range": "± 3016483",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 2135355932,
            "range": "± 6620704",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 4100554525,
            "range": "± 87531364",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 7994624643,
            "range": "± 70432114",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 3112562,
            "range": "± 44924",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 4550738,
            "range": "± 60078",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 7601018,
            "range": "± 88265",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 12172162,
            "range": "± 149562",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 21794841,
            "range": "± 180568",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 38241497,
            "range": "± 2762311",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 71578971,
            "range": "± 1944289",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 128204826,
            "range": "± 1796357",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 240059938,
            "range": "± 3020460",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-orchard-shaped/actions/1",
            "value": 13385546,
            "range": "± 821941",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-orchard-shaped/actions/2",
            "value": 14578066,
            "range": "± 1403988",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-orchard-shaped/actions/16",
            "value": 30507689,
            "range": "± 376378",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-orchard-shaped/actions/64",
            "value": 85339872,
            "range": "± 842390",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}