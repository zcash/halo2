window.BENCHMARK_DATA = {
  "lastUpdate": 1678213795738,
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
          "id": "2016154118a583d9736e079957b4886a13e94104",
          "message": "Merge pull request #746 from zcash/test-32-bit\n\nCI: Run tests on 32-bit target",
          "timestamp": "2023-03-07T17:26:05Z",
          "tree_id": "11b34856d3b6a72abe9f073751d3ba0439455ae6",
          "url": "https://github.com/zcash/halo2/commit/2016154118a583d9736e079957b4886a13e94104"
        },
        "date": 1678213775527,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 98487058,
            "range": "± 9164049",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 5663930,
            "range": "± 557625",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 181558487,
            "range": "± 6429675",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6748521,
            "range": "± 555510",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 248833828,
            "range": "± 8761846",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7500446,
            "range": "± 805336",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 46366,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 160815,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 175970,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 288074,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 288071,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 164259,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 179364,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 291433,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 291396,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 343578,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 358524,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 470478,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 470433,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3402768,
            "range": "± 3637",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5912895,
            "range": "± 7754",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10167969,
            "range": "± 310554",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21775697,
            "range": "± 332318",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41003826,
            "range": "± 320066",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 78657408,
            "range": "± 249622",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 6907,
            "range": "± 788",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8203,
            "range": "± 1194",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14937,
            "range": "± 1097",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19733,
            "range": "± 624",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26713,
            "range": "± 2372",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 44641,
            "range": "± 7299",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 102421,
            "range": "± 15591",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 218400,
            "range": "± 35746",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 478606,
            "range": "± 66766",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 923547,
            "range": "± 109943",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1686217,
            "range": "± 128346",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3358858,
            "range": "± 371354",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7959103,
            "range": "± 605810",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15610206,
            "range": "± 1962555",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 33653948,
            "range": "± 2178212",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 72912875,
            "range": "± 885819",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34510,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34640,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 173423812,
            "range": "± 1972682",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 374379787,
            "range": "± 4336179",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 811896398,
            "range": "± 7769219",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1733867772,
            "range": "± 10699871",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3695546718,
            "range": "± 9286026",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 7903993518,
            "range": "± 9533252",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 16841101483,
            "range": "± 14909752",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 35716457353,
            "range": "± 23873148",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 75519607449,
            "range": "± 56734607",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 117480777,
            "range": "± 4896477",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 206697465,
            "range": "± 5973489",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 351626757,
            "range": "± 4847699",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 621057470,
            "range": "± 7540050",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1128520260,
            "range": "± 15148476",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2111998166,
            "range": "± 15983889",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4040494519,
            "range": "± 7930402",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7746647010,
            "range": "± 28016640",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 14821555548,
            "range": "± 62430185",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5817396,
            "range": "± 464776",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9001888,
            "range": "± 616401",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14442282,
            "range": "± 765519",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24313124,
            "range": "± 1845195",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 41902344,
            "range": "± 2611266",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 74712491,
            "range": "± 4969907",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 134561056,
            "range": "± 5958570",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 247812734,
            "range": "± 16484431",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 450290369,
            "range": "± 6722425",
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
          "id": "a1034169bc13bf51aa9cbfe37043e4a195c97924",
          "message": "Merge pull request #743 from zcash/740-expose-instance-value\n\nhalo2_proofs: Add `Region::instance_value`",
          "timestamp": "2023-03-07T17:19:11Z",
          "tree_id": "82442cd86ca1be346adbba258cd8619529aa801a",
          "url": "https://github.com/zcash/halo2/commit/a1034169bc13bf51aa9cbfe37043e4a195c97924"
        },
        "date": 1678213786970,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 115706807,
            "range": "± 10764118",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 6479973,
            "range": "± 823961",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 212761437,
            "range": "± 12619743",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 7555881,
            "range": "± 935316",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 289550315,
            "range": "± 13489001",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 8413411,
            "range": "± 945119",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 52099,
            "range": "± 2965",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 173957,
            "range": "± 11665",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 185861,
            "range": "± 11620",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 309602,
            "range": "± 15817",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 299295,
            "range": "± 19388",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 180368,
            "range": "± 12845",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 189728,
            "range": "± 16558",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 307343,
            "range": "± 13101",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 317418,
            "range": "± 10020",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 381445,
            "range": "± 14119",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 387153,
            "range": "± 11137",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 511351,
            "range": "± 20195",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 506087,
            "range": "± 30999",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3698735,
            "range": "± 149127",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7833248,
            "range": "± 128990",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 13683906,
            "range": "± 240358",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 30313277,
            "range": "± 510740",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 55726663,
            "range": "± 1055076",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 109225460,
            "range": "± 3604744",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8548,
            "range": "± 989",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10249,
            "range": "± 2385",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18378,
            "range": "± 1540",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 23709,
            "range": "± 2461",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 32333,
            "range": "± 3886",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 56084,
            "range": "± 11749",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 133771,
            "range": "± 26487",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 281291,
            "range": "± 66542",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 560119,
            "range": "± 76712",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 995816,
            "range": "± 143240",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2079938,
            "range": "± 216613",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4457731,
            "range": "± 470119",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9649224,
            "range": "± 1185042",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21306539,
            "range": "± 2760879",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 47533050,
            "range": "± 3603264",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 95343571,
            "range": "± 5862873",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34504,
            "range": "± 1693",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34708,
            "range": "± 1758",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 195033875,
            "range": "± 8833864",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 417263021,
            "range": "± 8408556",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 854687364,
            "range": "± 33169458",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1844407380,
            "range": "± 50310408",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4004881154,
            "range": "± 86877889",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8584829335,
            "range": "± 97895333",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 18519952394,
            "range": "± 261451718",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 39008123463,
            "range": "± 410432496",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 84988394064,
            "range": "± 1291470078",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 133951334,
            "range": "± 5932195",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 235571529,
            "range": "± 8124662",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 396713695,
            "range": "± 14193533",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 712532837,
            "range": "± 19115555",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1252784550,
            "range": "± 16596978",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2446824550,
            "range": "± 57866559",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4525108361,
            "range": "± 52467228",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8649196707,
            "range": "± 146827783",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16661318059,
            "range": "± 317896764",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6388234,
            "range": "± 663277",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10011233,
            "range": "± 1216516",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16703493,
            "range": "± 2140160",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 26872545,
            "range": "± 2718159",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 46590171,
            "range": "± 3845213",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 85609890,
            "range": "± 7255216",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 152786264,
            "range": "± 8880002",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 292948342,
            "range": "± 21710241",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 512253923,
            "range": "± 20185039",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}