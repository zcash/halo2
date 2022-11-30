window.BENCHMARK_DATA = {
  "lastUpdate": 1669839702410,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "distinct": true,
          "id": "0ba0e40b874129e586fec0336c08011ef7e049a3",
          "message": "CI: Avoid testing against MSRV with test-dev-graph feature flag\n\nWe only need it for generating images of halo2_gadgets chips, and its\ntransitive dependencies have bumped MSRV in point releases.",
          "timestamp": "2022-11-24T01:56:54Z",
          "tree_id": "2f32fcd9ef72a5ac8a509e640656285eeacb6f0a",
          "url": "https://github.com/zcash/halo2/commit/0ba0e40b874129e586fec0336c08011ef7e049a3"
        },
        "date": 1669259377832,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 73712399,
            "range": "± 8268325",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3812091,
            "range": "± 246017",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 153831764,
            "range": "± 5579720",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4633189,
            "range": "± 209585",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 224579710,
            "range": "± 10936617",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5283400,
            "range": "± 362813",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 55423,
            "range": "± 1833",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 182800,
            "range": "± 4957",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 198099,
            "range": "± 5646",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 327340,
            "range": "± 9406",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 328691,
            "range": "± 16833",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 187115,
            "range": "± 5895",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 203912,
            "range": "± 15915",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 330008,
            "range": "± 12219",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 327529,
            "range": "± 18905",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 398737,
            "range": "± 16661",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 412351,
            "range": "± 9291",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 502865,
            "range": "± 23784",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 486483,
            "range": "± 19573",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3734541,
            "range": "± 182885",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7362077,
            "range": "± 177580",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12796272,
            "range": "± 358761",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 25289890,
            "range": "± 625661",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 54493475,
            "range": "± 1826929",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 107618049,
            "range": "± 1181413",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8148,
            "range": "± 622",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9649,
            "range": "± 737",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18115,
            "range": "± 1078",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 21458,
            "range": "± 776",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33453,
            "range": "± 1599",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 58949,
            "range": "± 4463",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 124014,
            "range": "± 9763",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 252265,
            "range": "± 31531",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 500145,
            "range": "± 35477",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1060500,
            "range": "± 60029",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2298256,
            "range": "± 111953",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4986855,
            "range": "± 142683",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10615229,
            "range": "± 350266",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 22447160,
            "range": "± 889614",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 48436979,
            "range": "± 1873917",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 119307012,
            "range": "± 15050354",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 37265,
            "range": "± 1845",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 37203,
            "range": "± 1869",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 211096194,
            "range": "± 3363270",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 422831789,
            "range": "± 21755944",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 931086096,
            "range": "± 32133847",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1991118647,
            "range": "± 30410181",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4255674801,
            "range": "± 45597595",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 9036782489,
            "range": "± 42365992",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 19943693508,
            "range": "± 158893298",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 42818447755,
            "range": "± 269136383",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 90082282819,
            "range": "± 1936418332",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 122989690,
            "range": "± 2707407",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 221960265,
            "range": "± 6215525",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 391813031,
            "range": "± 9340468",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 715881328,
            "range": "± 10906971",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1334675646,
            "range": "± 8853015",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2491743759,
            "range": "± 36445898",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4563398206,
            "range": "± 110255246",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8902199880,
            "range": "± 74675613",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 17366978256,
            "range": "± 291702539",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6696913,
            "range": "± 293822",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10414785,
            "range": "± 672369",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16608758,
            "range": "± 1016006",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 28497625,
            "range": "± 1901471",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 50563534,
            "range": "± 1907551",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 89906687,
            "range": "± 2852192",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 143764977,
            "range": "± 7356938",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 269877860,
            "range": "± 9355335",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 486802905,
            "range": "± 22671905",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "627b35528d244058d25e1d39d4094b9a1598a076",
          "message": "Merge pull request #688 from zhiqiangxu/fix_typo\n\nfix a typo",
          "timestamp": "2022-11-29T18:18:18Z",
          "tree_id": "475a12718f7428b4fdb97f08946d0d655713d199",
          "url": "https://github.com/zcash/halo2/commit/627b35528d244058d25e1d39d4094b9a1598a076"
        },
        "date": 1669749769316,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71952547,
            "range": "± 6967172",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3788288,
            "range": "± 364373",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 143228247,
            "range": "± 3338280",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4754889,
            "range": "± 638302",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 200330558,
            "range": "± 2576586",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5665406,
            "range": "± 572856",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47428,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164986,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 180379,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 291919,
            "range": "± 527",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 291784,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 168469,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 183892,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 295253,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 295337,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 352254,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 367811,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 479126,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 478881,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3706227,
            "range": "± 17105",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6087910,
            "range": "± 2911",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10375470,
            "range": "± 14413",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21572431,
            "range": "± 13554",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 40189325,
            "range": "± 269053",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 79910558,
            "range": "± 259452",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7296,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8516,
            "range": "± 1495",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16198,
            "range": "± 753",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20675,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 29129,
            "range": "± 1878",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 49121,
            "range": "± 8652",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 115865,
            "range": "± 15890",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 246462,
            "range": "± 38741",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 530167,
            "range": "± 65449",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 950360,
            "range": "± 121468",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1903013,
            "range": "± 79970",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3951903,
            "range": "± 275833",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8157167,
            "range": "± 796955",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21542521,
            "range": "± 2330638",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 39935523,
            "range": "± 3345185",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 82061033,
            "range": "± 1208096",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34789,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35061,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 177140236,
            "range": "± 628914",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 389039790,
            "range": "± 4798169",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 834858726,
            "range": "± 12250079",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1769364765,
            "range": "± 9035638",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3784727018,
            "range": "± 8925730",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8090858121,
            "range": "± 10387229",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17256911319,
            "range": "± 55169779",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36593657898,
            "range": "± 104819376",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77783834830,
            "range": "± 240225839",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 121279394,
            "range": "± 4707392",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 208093258,
            "range": "± 4604175",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 363639190,
            "range": "± 8542865",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 635428827,
            "range": "± 11982069",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1155153067,
            "range": "± 16148643",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2165148709,
            "range": "± 13590986",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4110538091,
            "range": "± 7826577",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7893790689,
            "range": "± 21817516",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15119746584,
            "range": "± 30862839",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5973019,
            "range": "± 723406",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9255805,
            "range": "± 926056",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14676945,
            "range": "± 1185965",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24873553,
            "range": "± 2130161",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42794701,
            "range": "± 2667692",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76517939,
            "range": "± 5334384",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137708467,
            "range": "± 5957573",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 254958364,
            "range": "± 7934801",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 459383128,
            "range": "± 6432935",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "733b4f356752166645be42c3ee7d8644877239c9",
          "message": "Merge pull request #694 from zcash/simplify-evaluationdomain\n\nSimplify `EvaluationDomain` to only accept field elements",
          "timestamp": "2022-11-29T18:39:46Z",
          "tree_id": "c3a7c001b6a908bcb723026e455f63b1e083910f",
          "url": "https://github.com/zcash/halo2/commit/733b4f356752166645be42c3ee7d8644877239c9"
        },
        "date": 1669751068044,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72671184,
            "range": "± 5821104",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3906983,
            "range": "± 399274",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 143699406,
            "range": "± 2868663",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4765048,
            "range": "± 540138",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 200280151,
            "range": "± 2299712",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5747020,
            "range": "± 543807",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47114,
            "range": "± 1089",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164638,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 179816,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 292785,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 292719,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 168037,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 183286,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 296105,
            "range": "± 1097",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 296057,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 351878,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 366800,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 480155,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 479845,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3634438,
            "range": "± 17271",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6049337,
            "range": "± 7547",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10651516,
            "range": "± 35633",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 23055707,
            "range": "± 109772",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42040540,
            "range": "± 890437",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 81303978,
            "range": "± 219550",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7198,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8537,
            "range": "± 1414",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16022,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19854,
            "range": "± 894",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 29025,
            "range": "± 3237",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 49649,
            "range": "± 8396",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 117931,
            "range": "± 18242",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 252786,
            "range": "± 43527",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 531921,
            "range": "± 70920",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 975538,
            "range": "± 112444",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1863885,
            "range": "± 79476",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3980836,
            "range": "± 300586",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8223316,
            "range": "± 745311",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21821917,
            "range": "± 2185441",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 44013548,
            "range": "± 3106630",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 83365377,
            "range": "± 995747",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34793,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35077,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 180267691,
            "range": "± 8605471",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 392016857,
            "range": "± 6185897",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 831929223,
            "range": "± 8804879",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1789658182,
            "range": "± 10469180",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3806706326,
            "range": "± 8753833",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8125967123,
            "range": "± 22246571",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17322914172,
            "range": "± 33706139",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36733143138,
            "range": "± 87456005",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77683086657,
            "range": "± 60962579",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 116662095,
            "range": "± 3837811",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 213421079,
            "range": "± 7356100",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 359901431,
            "range": "± 7845391",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 637386986,
            "range": "± 16985491",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1161190362,
            "range": "± 13846986",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2160692883,
            "range": "± 15220162",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4101975222,
            "range": "± 21716042",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7909618255,
            "range": "± 28876341",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15151827979,
            "range": "± 48704937",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5983190,
            "range": "± 625561",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9275067,
            "range": "± 891795",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14768527,
            "range": "± 1022442",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24907761,
            "range": "± 2272911",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42860242,
            "range": "± 3375292",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76577210,
            "range": "± 5254666",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 138146258,
            "range": "± 6406321",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 253124874,
            "range": "± 6205331",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 461037229,
            "range": "± 6427975",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "b097f098ae2cab0957dd3edf8b1a9ca823c46451",
          "message": "Merge pull request #685 from zcash/zero-area-region\n\n[MockProver] failure::FailureLocation: Handle zero-area regions.",
          "timestamp": "2022-11-29T19:26:12Z",
          "tree_id": "e125b439b174878e10a337be0cfbaa136a956667",
          "url": "https://github.com/zcash/halo2/commit/b097f098ae2cab0957dd3edf8b1a9ca823c46451"
        },
        "date": 1669753825575,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71475899,
            "range": "± 7858221",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3817671,
            "range": "± 380697",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 143504128,
            "range": "± 5022416",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4579762,
            "range": "± 615473",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 200794103,
            "range": "± 2049307",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5675756,
            "range": "± 526518",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47044,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164543,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 179765,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 289136,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 289197,
            "range": "± 932",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 168062,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 183219,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 292479,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 292571,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 351904,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 366777,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 476321,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 476118,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3546831,
            "range": "± 1300",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6015879,
            "range": "± 3202",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10764037,
            "range": "± 57696",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21413416,
            "range": "± 66362",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 40153553,
            "range": "± 229786",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 79012983,
            "range": "± 183173",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7152,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8309,
            "range": "± 1333",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16396,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20566,
            "range": "± 1492",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 28263,
            "range": "± 3649",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 49326,
            "range": "± 8973",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 118490,
            "range": "± 17298",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 243557,
            "range": "± 43381",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 532915,
            "range": "± 68153",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 968798,
            "range": "± 105427",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1846700,
            "range": "± 110599",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3957919,
            "range": "± 270375",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8100732,
            "range": "± 499498",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21548060,
            "range": "± 2076004",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 37234111,
            "range": "± 1950375",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 82395660,
            "range": "± 1270424",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34784,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35065,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 177815855,
            "range": "± 1607391",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 389540484,
            "range": "± 3873043",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 826854309,
            "range": "± 8976974",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1772361331,
            "range": "± 12157951",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3792129495,
            "range": "± 6228500",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8102662120,
            "range": "± 6528441",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17257523255,
            "range": "± 27647807",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36604372740,
            "range": "± 22763632",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77419538372,
            "range": "± 126494936",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 120499791,
            "range": "± 4951411",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 209472462,
            "range": "± 6425689",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 364408254,
            "range": "± 8398776",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 637117327,
            "range": "± 10819478",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1159535611,
            "range": "± 8296524",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2163686978,
            "range": "± 21366736",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4114003726,
            "range": "± 12926963",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7916009814,
            "range": "± 16234526",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15097299807,
            "range": "± 50922815",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5924499,
            "range": "± 669157",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9184841,
            "range": "± 732627",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14833081,
            "range": "± 1094299",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24808025,
            "range": "± 1573757",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42677739,
            "range": "± 2382627",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76417938,
            "range": "± 5288557",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137698683,
            "range": "± 5338447",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 252594177,
            "range": "± 6260738",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 462849406,
            "range": "± 5800685",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "a19ce33c395eb14f951e4d64d1bd3c7d6f714366",
          "message": "Merge pull request #696 from zcash/fieldext-final-cleanups\n\nFinal cleanups before `FieldExt` removal",
          "timestamp": "2022-11-30T05:01:33Z",
          "tree_id": "79bc7f85d610d7e6e99ffc76c49b18dbcd83f8dc",
          "url": "https://github.com/zcash/halo2/commit/a19ce33c395eb14f951e4d64d1bd3c7d6f714366"
        },
        "date": 1669788351778,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72358484,
            "range": "± 4803073",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3803909,
            "range": "± 433335",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 143546718,
            "range": "± 3348984",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4785508,
            "range": "± 604968",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 201369598,
            "range": "± 3644712",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5555953,
            "range": "± 577390",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47087,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164530,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 179792,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 290890,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 290883,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 168014,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 183343,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 294360,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 294314,
            "range": "± 15070",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 351515,
            "range": "± 957",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 366697,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 477747,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 477870,
            "range": "± 447",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3651354,
            "range": "± 1967",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6048840,
            "range": "± 22535",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10439971,
            "range": "± 9941",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 23020953,
            "range": "± 63005",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42100203,
            "range": "± 422250",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 80960361,
            "range": "± 223458",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7224,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8444,
            "range": "± 717",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14895,
            "range": "± 817",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18173,
            "range": "± 843",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26263,
            "range": "± 2761",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 43986,
            "range": "± 6368",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 101840,
            "range": "± 14404",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 210839,
            "range": "± 35235",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 462319,
            "range": "± 61603",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 856287,
            "range": "± 92740",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1676123,
            "range": "± 126441",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3377577,
            "range": "± 340952",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7749051,
            "range": "± 688474",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15222100,
            "range": "± 1527760",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 37731034,
            "range": "± 3047067",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 72665619,
            "range": "± 3886266",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34813,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35107,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 177001187,
            "range": "± 992665",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 391955439,
            "range": "± 3454582",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 830025750,
            "range": "± 11998977",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1773029247,
            "range": "± 10979758",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3778928453,
            "range": "± 11014812",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8090036312,
            "range": "± 8141783",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17210028715,
            "range": "± 23937273",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36501115130,
            "range": "± 24063309",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77140622207,
            "range": "± 76893747",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 119981207,
            "range": "± 2778542",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 210416039,
            "range": "± 4025726",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 358994703,
            "range": "± 5716689",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 635593792,
            "range": "± 16923894",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1143967418,
            "range": "± 8376099",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2157950692,
            "range": "± 8591596",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4123034830,
            "range": "± 30062141",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7891367666,
            "range": "± 35423472",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15088595930,
            "range": "± 47519208",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5919622,
            "range": "± 667564",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9159056,
            "range": "± 616390",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14685392,
            "range": "± 768257",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24811209,
            "range": "± 1929673",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42787599,
            "range": "± 2851172",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76557314,
            "range": "± 4005775",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137460765,
            "range": "± 5027808",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 253243949,
            "range": "± 7247160",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 460310023,
            "range": "± 5967561",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "455ee88b96387477b7343615eebe9e83208aae95",
          "message": "Merge pull request #687 from naure/optim/linear-multiopen\n\nOptimize multiopen verifier for many columns",
          "timestamp": "2022-11-30T07:25:56Z",
          "tree_id": "f1cde3665194fa2ef712fcf152c46ccb906e6683",
          "url": "https://github.com/zcash/halo2/commit/455ee88b96387477b7343615eebe9e83208aae95"
        },
        "date": 1669797247849,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 76809967,
            "range": "± 2916450",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3937700,
            "range": "± 444469",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 161833322,
            "range": "± 4135049",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5062810,
            "range": "± 713912",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 222057150,
            "range": "± 9051307",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6150884,
            "range": "± 539081",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 46937,
            "range": "± 3001",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 170190,
            "range": "± 2203",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 183420,
            "range": "± 2849",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 299227,
            "range": "± 3111",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 297587,
            "range": "± 6511",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 173084,
            "range": "± 2475",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 186211,
            "range": "± 3352",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 299746,
            "range": "± 4386",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 298966,
            "range": "± 4355",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 360918,
            "range": "± 4194",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 379479,
            "range": "± 4088",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 492273,
            "range": "± 3604",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 488933,
            "range": "± 4994",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3612372,
            "range": "± 37087",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7105416,
            "range": "± 98473",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12557602,
            "range": "± 111334",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 26828329,
            "range": "± 227214",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 51065004,
            "range": "± 360691",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 97548728,
            "range": "± 623562",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8724,
            "range": "± 417",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10033,
            "range": "± 459",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 17678,
            "range": "± 888",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 21070,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 30537,
            "range": "± 2038",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 50491,
            "range": "± 6804",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 115377,
            "range": "± 17164",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 247157,
            "range": "± 47513",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 520975,
            "range": "± 64385",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 943796,
            "range": "± 114633",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1929327,
            "range": "± 197813",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4019049,
            "range": "± 400259",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8461858,
            "range": "± 432060",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 18697741,
            "range": "± 1952105",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 41536484,
            "range": "± 1800546",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 89665103,
            "range": "± 2955014",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 33747,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 33629,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 199212235,
            "range": "± 15232012",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 413683324,
            "range": "± 5146452",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 881896762,
            "range": "± 10778860",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1913970149,
            "range": "± 28257039",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3886927594,
            "range": "± 113061960",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8610088029,
            "range": "± 158229492",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17998700285,
            "range": "± 356936293",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 38056000674,
            "range": "± 712297081",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 80078406958,
            "range": "± 844470459",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 130859035,
            "range": "± 5467173",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 225996207,
            "range": "± 8342118",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 379546081,
            "range": "± 9919842",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 699835384,
            "range": "± 15881065",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1221505412,
            "range": "± 11947001",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2328906602,
            "range": "± 40489038",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4443863911,
            "range": "± 87727390",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8598882724,
            "range": "± 159178377",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16448456154,
            "range": "± 112753700",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6339641,
            "range": "± 540885",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9594343,
            "range": "± 1079321",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15820673,
            "range": "± 1427065",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 26443481,
            "range": "± 2423144",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 48312983,
            "range": "± 5694888",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 82008284,
            "range": "± 6170333",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 144337115,
            "range": "± 6685820",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 273463718,
            "range": "± 9593746",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 497078425,
            "range": "± 18877022",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "6ae9f77e04d471c64b31b86486fb6ae974dc31a1",
          "message": "Merge pull request #678 from nuttycom/fix_clippy_lints\n\nFix beta and nightly clippy complaints",
          "timestamp": "2022-11-30T19:25:54Z",
          "tree_id": "0409f7ac95f5f76648afba66d055446b2dfad4e3",
          "url": "https://github.com/zcash/halo2/commit/6ae9f77e04d471c64b31b86486fb6ae974dc31a1"
        },
        "date": 1669839695202,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 65105240,
            "range": "± 6664583",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2967555,
            "range": "± 401227",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 133871229,
            "range": "± 8246255",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3995228,
            "range": "± 543653",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 181902448,
            "range": "± 8946657",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4014672,
            "range": "± 432486",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 39454,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 125887,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 136590,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 250674,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 221098,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 145636,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 139302,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 251596,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 253592,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 268756,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 316907,
            "range": "± 1140",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 413627,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 413591,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3021383,
            "range": "± 1787",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5235781,
            "range": "± 30231",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 9676547,
            "range": "± 52237",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21570375,
            "range": "± 205727",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 40029528,
            "range": "± 187088",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 76724697,
            "range": "± 226865",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7101,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8793,
            "range": "± 841",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14187,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18286,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26215,
            "range": "± 3191",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 42189,
            "range": "± 5165",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 88684,
            "range": "± 21027",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 203890,
            "range": "± 64712",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 428533,
            "range": "± 73791",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 807250,
            "range": "± 119305",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1437227,
            "range": "± 115064",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3141424,
            "range": "± 372573",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 6566057,
            "range": "± 400540",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 14969468,
            "range": "± 1501201",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 31607085,
            "range": "± 2419696",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 76992693,
            "range": "± 2362241",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 25251,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28639,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 146587022,
            "range": "± 8851952",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 329562814,
            "range": "± 12789926",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 717426010,
            "range": "± 48371493",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1525219360,
            "range": "± 62057726",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 2967315772,
            "range": "± 184791981",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6889851043,
            "range": "± 450817438",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 13470112194,
            "range": "± 662819105",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 28475478416,
            "range": "± 1490483053",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 60147989896,
            "range": "± 3364179434",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 97243956,
            "range": "± 6231099",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 171896875,
            "range": "± 10515844",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 307068050,
            "range": "± 10372935",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 523828538,
            "range": "± 24954143",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 949380862,
            "range": "± 50895981",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1821888137,
            "range": "± 102629594",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3335560657,
            "range": "± 187583598",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6675061636,
            "range": "± 245060780",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 12658782131,
            "range": "± 601692268",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5179061,
            "range": "± 247752",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7955109,
            "range": "± 230531",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12852451,
            "range": "± 521572",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21824161,
            "range": "± 1353225",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37519776,
            "range": "± 3555220",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66473457,
            "range": "± 3775647",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 106820773,
            "range": "± 7468293",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 221739844,
            "range": "± 11278624",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 402923587,
            "range": "± 21812325",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}