window.BENCHMARK_DATA = {
  "lastUpdate": 1669751075338,
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
      }
    ]
  }
}