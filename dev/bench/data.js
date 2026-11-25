window.BENCHMARK_DATA = {
  "lastUpdate": 1795577815174,
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
      },
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
          "id": "7465edf9110aace00b8c24b96270e1b820fd39fb",
          "message": "Merge pull request #984 from upbqdn/upstream/verifier-scalar\n\nReduce verifier scalar work and MSM allocation",
          "timestamp": "2026-11-14T12:38:35+01:00",
          "tree_id": "d4ac533637f6338a1da75ba867ff8d03f825740c",
          "url": "https://github.com/zcash/halo2/commit/7465edf9110aace00b8c24b96270e1b820fd39fb"
        },
        "date": 1794658769640,
        "tool": "cargo",
        "benches": [
          {
            "name": "plonk-verifier-batch/proofs/1",
            "value": 6989680,
            "range": "± 255134",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-batch/proofs/8",
            "value": 11521688,
            "range": "± 186099",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-batch/proofs/64",
            "value": 48292019,
            "range": "± 702001",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jack@zodl.com",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9fdacc869841cc3d7a1bf4e5d01b53b752c1d1bc",
          "message": "Merge pull request #989 from upbqdn/upstream/pasta-msm-hook\n\nDispatch public MSMs through the Pasta curve hook",
          "timestamp": "2026-11-25T02:55:43Z",
          "tree_id": "cd2d8d335a0f9d7c7c94e2ee0a09e169f4540e37",
          "url": "https://github.com/zcash/halo2/commit/9fdacc869841cc3d7a1bf4e5d01b53b752c1d1bc"
        },
        "date": 1795577807800,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 59961719,
            "range": "± 2713825",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 1414828,
            "range": "± 18995",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 108885459,
            "range": "± 2671922",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 1704409,
            "range": "± 19345",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 138199527,
            "range": "± 1529738",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 1864511,
            "range": "± 17042",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 25566,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 102103,
            "range": "± 921",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 103625,
            "range": "± 1947",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 182932,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 182968,
            "range": "± 655",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 102918,
            "range": "± 894",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 104623,
            "range": "± 1152",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 185015,
            "range": "± 1014",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 185083,
            "range": "± 2034",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 215344,
            "range": "± 1226",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 217208,
            "range": "± 1285",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 297617,
            "range": "± 974",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 297692,
            "range": "± 1647",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2310706,
            "range": "± 8127",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 3606330,
            "range": "± 10773",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 6504243,
            "range": "± 30813",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 15631719,
            "range": "± 118739",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 30675665,
            "range": "± 56268",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 60517534,
            "range": "± 201762",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 6092,
            "range": "± 941",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 7938,
            "range": "± 527",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 9982,
            "range": "± 500",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 14895,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 23505,
            "range": "± 1772",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 48512,
            "range": "± 1476",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 83397,
            "range": "± 1313",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 149775,
            "range": "± 1891",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 280778,
            "range": "± 21022",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 546763,
            "range": "± 5176",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1114396,
            "range": "± 19570",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 2297329,
            "range": "± 32944",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 4845601,
            "range": "± 69852",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 10391377,
            "range": "± 132735",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 22497189,
            "range": "± 363681",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 49719748,
            "range": "± 1219973",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 19447,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 19517,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/generic/1",
            "value": 142401,
            "range": "± 2139",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/hooked/1",
            "value": 49615,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/generic/2",
            "value": 189325,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/hooked/2",
            "value": 71317,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/generic/8",
            "value": 1272914,
            "range": "± 10796",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/hooked/8",
            "value": 194397,
            "range": "± 4042",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/generic/16",
            "value": 1410296,
            "range": "± 6067",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/hooked/16",
            "value": 323725,
            "range": "± 10195",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/generic/32",
            "value": 1302404,
            "range": "± 8627",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/hooked/32",
            "value": 570992,
            "range": "± 10796",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/generic/64",
            "value": 1460692,
            "range": "± 9557",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/hooked/64",
            "value": 559377,
            "range": "± 5006",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/generic/2048",
            "value": 12422330,
            "range": "± 64546",
            "unit": "ns/iter"
          },
          {
            "name": "msm/pallas/hooked/2048",
            "value": 6586945,
            "range": "± 68257",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/generic/1",
            "value": 141089,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/hooked/1",
            "value": 50544,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/generic/2",
            "value": 183968,
            "range": "± 1601",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/hooked/2",
            "value": 71358,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/generic/8",
            "value": 1328531,
            "range": "± 39927",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/hooked/8",
            "value": 196841,
            "range": "± 4321",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/generic/16",
            "value": 1416209,
            "range": "± 15008",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/hooked/16",
            "value": 310091,
            "range": "± 10166",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/generic/32",
            "value": 1322308,
            "range": "± 18351",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/hooked/32",
            "value": 529936,
            "range": "± 24921",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/generic/64",
            "value": 1418423,
            "range": "± 21369",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/hooked/64",
            "value": 558119,
            "range": "± 4630",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/generic/2048",
            "value": 12412153,
            "range": "± 55396",
            "unit": "ns/iter"
          },
          {
            "name": "msm/vesta/hooked/2048",
            "value": 6531914,
            "range": "± 44284",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/8",
            "value": 2737781,
            "range": "± 63211",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/9",
            "value": 4242721,
            "range": "± 216034",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/10",
            "value": 7471723,
            "range": "± 75282",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/11",
            "value": 12426058,
            "range": "± 70375",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/12",
            "value": 22631272,
            "range": "± 78514",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/13",
            "value": 40284882,
            "range": "± 588746",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/14",
            "value": 76561505,
            "range": "± 1445952",
            "unit": "ns/iter"
          },
          {
            "name": "msm/k/15",
            "value": 137202285,
            "range": "± 491797",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 50930050,
            "range": "± 614454",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 101965196,
            "range": "± 1373144",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 208364123,
            "range": "± 2872033",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 432671349,
            "range": "± 6147942",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 902295393,
            "range": "± 13063282",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 1904709853,
            "range": "± 36660422",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 4071017635,
            "range": "± 68582346",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 8443794873,
            "range": "± 159416367",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 17559529366,
            "range": "± 305688288",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 59314881,
            "range": "± 217297",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 90415272,
            "range": "± 265564",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 151676422,
            "range": "± 790304",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 257862622,
            "range": "± 1752683",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 465656473,
            "range": "± 6304874",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 854317570,
            "range": "± 7794742",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 1626509423,
            "range": "± 17632105",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 3113159973,
            "range": "± 35066289",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 5783529051,
            "range": "± 36520238",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 1759379,
            "range": "± 16252",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 2599434,
            "range": "± 18432",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 4095622,
            "range": "± 26912",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 7069127,
            "range": "± 44235",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 13583442,
            "range": "± 117311",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 22838764,
            "range": "± 181308",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 42885917,
            "range": "± 443712",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 81959186,
            "range": "± 4820633",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 159377838,
            "range": "± 2093216",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-orchard-shaped/actions/1",
            "value": 7130102,
            "range": "± 61530",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-orchard-shaped/actions/2",
            "value": 7221514,
            "range": "± 94296",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-orchard-shaped/actions/16",
            "value": 8676052,
            "range": "± 76998",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-orchard-shaped/actions/64",
            "value": 13784538,
            "range": "± 109490",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-batch/proofs/1",
            "value": 7130390,
            "range": "± 53256",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-batch/proofs/8",
            "value": 8150185,
            "range": "± 95094",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier-batch/proofs/64",
            "value": 16878445,
            "range": "± 700454",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}