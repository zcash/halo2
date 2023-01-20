window.BENCHMARK_DATA = {
  "lastUpdate": 1674252057731,
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
          "id": "d771a48f686af0a65f5171905b8b855e55774bd1",
          "message": "Merge pull request #718 from HollowMan6/main\n\nFix book deploying CI and add pdf uploading",
          "timestamp": "2023-01-20T20:52:28Z",
          "tree_id": "3d7911ccfaf47965579d79c55ccef2ec11fcefc7",
          "url": "https://github.com/zcash/halo2/commit/d771a48f686af0a65f5171905b8b855e55774bd1"
        },
        "date": 1674251631062,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 73740172,
            "range": "± 7239544",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3479011,
            "range": "± 385453",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 147544183,
            "range": "± 7678784",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4066359,
            "range": "± 495993",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 195531850,
            "range": "± 6891199",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5010145,
            "range": "± 711265",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 40321,
            "range": "± 2740",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 145920,
            "range": "± 8881",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 155556,
            "range": "± 11014",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 263683,
            "range": "± 17354",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 277370,
            "range": "± 14344",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 150291,
            "range": "± 9340",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 171512,
            "range": "± 8490",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 262407,
            "range": "± 13230",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 264388,
            "range": "± 14965",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 315738,
            "range": "± 17504",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 324113,
            "range": "± 21028",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 433228,
            "range": "± 23016",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 441956,
            "range": "± 24292",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3140291,
            "range": "± 164993",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6537749,
            "range": "± 374964",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10855090,
            "range": "± 214710",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 23628065,
            "range": "± 501829",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43009943,
            "range": "± 880961",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 88379517,
            "range": "± 3023147",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8218,
            "range": "± 857",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9748,
            "range": "± 2415",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16314,
            "range": "± 769",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19824,
            "range": "± 1048",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27902,
            "range": "± 2888",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 46581,
            "range": "± 6912",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 109022,
            "range": "± 15917",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 213515,
            "range": "± 42581",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 494755,
            "range": "± 70705",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 899636,
            "range": "± 107832",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1701245,
            "range": "± 157789",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3690720,
            "range": "± 400380",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7575680,
            "range": "± 469238",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 16300122,
            "range": "± 964245",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 36855055,
            "range": "± 1655986",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 83532428,
            "range": "± 3454172",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 29457,
            "range": "± 1710",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 30811,
            "range": "± 1741",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 169113247,
            "range": "± 10123170",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 362495011,
            "range": "± 8076985",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 770797845,
            "range": "± 28196529",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1671149332,
            "range": "± 44786589",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3557550657,
            "range": "± 92135215",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 7682556278,
            "range": "± 166439554",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 15977101922,
            "range": "± 345761459",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 33977038208,
            "range": "± 321198572",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 72034807458,
            "range": "± 648556738",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 113431328,
            "range": "± 5393781",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 198427517,
            "range": "± 7146797",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 337500037,
            "range": "± 8411265",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 613121892,
            "range": "± 27138008",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1084167014,
            "range": "± 39867484",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2047570747,
            "range": "± 39786223",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3884807430,
            "range": "± 136724103",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7422166022,
            "range": "± 96130193",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 14702661892,
            "range": "± 270764155",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5703017,
            "range": "± 601135",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8656962,
            "range": "± 988387",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 13960924,
            "range": "± 1336306",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 23931492,
            "range": "± 2009082",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 41693090,
            "range": "± 2826494",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 73471090,
            "range": "± 4079080",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 132452801,
            "range": "± 7739169",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 245556147,
            "range": "± 19177343",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 442784182,
            "range": "± 16503100",
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
          "id": "1b1fdc2e2d946855852d741c3c2e0ce56ed6e7ef",
          "message": "Merge pull request #721 from daira/safari-rayon-status\n\nUpdate the status of support for nested web workers in Safari",
          "timestamp": "2023-01-20T20:48:51Z",
          "tree_id": "2c6d5a08c735e0c80784abb7b8d3185565fec780",
          "url": "https://github.com/zcash/halo2/commit/1b1fdc2e2d946855852d741c3c2e0ce56ed6e7ef"
        },
        "date": 1674252047923,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 79747895,
            "range": "± 5528513",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4144117,
            "range": "± 558202",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 162211416,
            "range": "± 5737565",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5737122,
            "range": "± 726546",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 223821889,
            "range": "± 8614453",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6278964,
            "range": "± 701126",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 54125,
            "range": "± 2245",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 183249,
            "range": "± 10606",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 197533,
            "range": "± 14901",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 326362,
            "range": "± 8368",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 326351,
            "range": "± 11049",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 186392,
            "range": "± 6613",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 201728,
            "range": "± 6916",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 330367,
            "range": "± 9384",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 329752,
            "range": "± 7297",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 389859,
            "range": "± 16223",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 405003,
            "range": "± 11933",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 540747,
            "range": "± 31093",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 534789,
            "range": "± 17250",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3858071,
            "range": "± 122435",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7391853,
            "range": "± 69203",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12914354,
            "range": "± 296353",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 27112457,
            "range": "± 466224",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 51042541,
            "range": "± 877247",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 101232705,
            "range": "± 1284182",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7659,
            "range": "± 726",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9997,
            "range": "± 2203",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 17300,
            "range": "± 1105",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 21626,
            "range": "± 1831",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 30323,
            "range": "± 3607",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 53304,
            "range": "± 11796",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 127019,
            "range": "± 20547",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 267061,
            "range": "± 44694",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 565600,
            "range": "± 84246",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1014394,
            "range": "± 121067",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2022514,
            "range": "± 167838",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4544938,
            "range": "± 522388",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10632074,
            "range": "± 1083818",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21923315,
            "range": "± 2391015",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 44481956,
            "range": "± 2854206",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 96883716,
            "range": "± 4135595",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 36015,
            "range": "± 1293",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 36237,
            "range": "± 1391",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 205518876,
            "range": "± 11014244",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 445393067,
            "range": "± 6509435",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 956688254,
            "range": "± 11184985",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 2057013496,
            "range": "± 19311523",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4387920749,
            "range": "± 50858380",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 9337449285,
            "range": "± 59108095",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 19900080436,
            "range": "± 50898273",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 42132105732,
            "range": "± 81022986",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 89054563796,
            "range": "± 100341284",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 137235141,
            "range": "± 5244614",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 236708225,
            "range": "± 7398472",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 409212913,
            "range": "± 9911619",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 738675750,
            "range": "± 16400997",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1329359271,
            "range": "± 18493472",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2532386041,
            "range": "± 24308370",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4773281690,
            "range": "± 40407710",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 9148966595,
            "range": "± 62891404",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 17578514762,
            "range": "± 65326198",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6818152,
            "range": "± 723401",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10660144,
            "range": "± 1051454",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 17123228,
            "range": "± 2020333",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 29031546,
            "range": "± 2924103",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 50357438,
            "range": "± 5451059",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 89104798,
            "range": "± 5878303",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 163276052,
            "range": "± 12387952",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 303845237,
            "range": "± 7756976",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 552784509,
            "range": "± 12520656",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}