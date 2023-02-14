window.BENCHMARK_DATA = {
  "lastUpdate": 1676410213279,
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
          "id": "47f2cc83498de27bcf1641e9f2e02e0c86551c7e",
          "message": "Merge pull request #728 from nagatoism/disable-rayon\n\n  Add feature \"multicore\"  and you can disable rayon by disabling the  \"multicore\" feature.",
          "timestamp": "2023-02-14T20:32:28Z",
          "tree_id": "cb509f92f5167939dd6a145ff5db146427fc191c",
          "url": "https://github.com/zcash/halo2/commit/47f2cc83498de27bcf1641e9f2e02e0c86551c7e"
        },
        "date": 1676410203398,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 92034344,
            "range": "± 7852661",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4629745,
            "range": "± 498151",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 167972641,
            "range": "± 3048995",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5494044,
            "range": "± 482933",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 228184787,
            "range": "± 5775126",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6452316,
            "range": "± 689957",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 35363,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 139465,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 151799,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 248537,
            "range": "± 480",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 248495,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 142520,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 154693,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 251256,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 251214,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 298029,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 310180,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 406890,
            "range": "± 3116",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 406858,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2960488,
            "range": "± 3147",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5866189,
            "range": "± 4155",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10281493,
            "range": "± 50180",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22835278,
            "range": "± 214152",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43123269,
            "range": "± 317163",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 82876189,
            "range": "± 589180",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7391,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8739,
            "range": "± 1417",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14601,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18607,
            "range": "± 551",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26127,
            "range": "± 1759",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 42001,
            "range": "± 6453",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 89577,
            "range": "± 12745",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 202145,
            "range": "± 36258",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 427030,
            "range": "± 69428",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 831991,
            "range": "± 101506",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1614952,
            "range": "± 161679",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3273393,
            "range": "± 362005",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 6912562,
            "range": "± 355180",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15063368,
            "range": "± 607628",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 33629863,
            "range": "± 1635015",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 76543822,
            "range": "± 3657388",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28567,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28702,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153264368,
            "range": "± 4248265",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 325687796,
            "range": "± 1492235",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 706584753,
            "range": "± 10542852",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1524087313,
            "range": "± 7754871",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3238336133,
            "range": "± 8653820",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6922650092,
            "range": "± 9544865",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14738072866,
            "range": "± 15507661",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31291149153,
            "range": "± 37962321",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66154544536,
            "range": "± 52919233",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 103475222,
            "range": "± 9278466",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 182234323,
            "range": "± 7732989",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 315966054,
            "range": "± 7109450",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 555442302,
            "range": "± 13606946",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1001156739,
            "range": "± 11204231",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1875592503,
            "range": "± 10149319",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3600242953,
            "range": "± 20522945",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6917380332,
            "range": "± 23441060",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13272161557,
            "range": "± 82472145",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5128089,
            "range": "± 70770",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7960150,
            "range": "± 588109",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12756659,
            "range": "± 818926",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21675567,
            "range": "± 1736028",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37375372,
            "range": "± 2005575",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66673171,
            "range": "± 4665994",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 119789351,
            "range": "± 5313309",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222496789,
            "range": "± 10286364",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 408307537,
            "range": "± 7133403",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}