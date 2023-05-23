window.BENCHMARK_DATA = {
  "lastUpdate": 1684867527963,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "yingtong.lai@gmail.com",
            "name": "ying tong",
            "username": "therealyingtong"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cf0a2d16d7563d013b00df2b03b903aaf3c32006",
          "message": "Merge pull request #762 from duguorong009/book-patch-user-dev-tools\n\n[book] Update `book/user/dev-tools.md`",
          "timestamp": "2023-05-24T01:31:54+08:00",
          "tree_id": "8775e97a6a37fa3427ae09c6dd36c8306a975bc8",
          "url": "https://github.com/zcash/halo2/commit/cf0a2d16d7563d013b00df2b03b903aaf3c32006"
        },
        "date": 1684866643333,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 91871338,
            "range": "± 8479562",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4837847,
            "range": "± 439646",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 168041683,
            "range": "± 4990402",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5425014,
            "range": "± 573056",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 232540542,
            "range": "± 9575109",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6933557,
            "range": "± 767249",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 39624,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 140659,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 152964,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 249931,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 250114,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 143656,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 155914,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 252760,
            "range": "± 614",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 252799,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 297978,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 310295,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 410088,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 409953,
            "range": "± 391",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3034638,
            "range": "± 2163",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5862906,
            "range": "± 13990",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10381906,
            "range": "± 59105",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 23635686,
            "range": "± 91291",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43196555,
            "range": "± 157110",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 84893578,
            "range": "± 141452",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7510,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8840,
            "range": "± 1348",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14954,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19169,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26215,
            "range": "± 1816",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 43433,
            "range": "± 8777",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 105077,
            "range": "± 18821",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 213948,
            "range": "± 43002",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 459463,
            "range": "± 62223",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 851159,
            "range": "± 114154",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1635074,
            "range": "± 101474",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3429703,
            "range": "± 373563",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7190427,
            "range": "± 666254",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15726503,
            "range": "± 1081472",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 34943086,
            "range": "± 2394798",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 80794846,
            "range": "± 3073453",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28595,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28704,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 154375943,
            "range": "± 7653336",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 328063559,
            "range": "± 4405826",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 719977577,
            "range": "± 6759656",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1515863522,
            "range": "± 10537653",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3252413489,
            "range": "± 6103818",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6952939453,
            "range": "± 26371829",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14816344545,
            "range": "± 23700121",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31437103691,
            "range": "± 80891368",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66492856412,
            "range": "± 103550587",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 101258262,
            "range": "± 4365242",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 184239648,
            "range": "± 7375621",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 315110921,
            "range": "± 6738691",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 556007752,
            "range": "± 10090452",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1018631680,
            "range": "± 10319478",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1911144311,
            "range": "± 23130257",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3609546207,
            "range": "± 25205368",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6962658744,
            "range": "± 22104612",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13347644677,
            "range": "± 77167573",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5148280,
            "range": "± 85852",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7925464,
            "range": "± 646220",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12758374,
            "range": "± 735458",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21676203,
            "range": "± 1816369",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37602240,
            "range": "± 2516745",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66985333,
            "range": "± 4903413",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 119949802,
            "range": "± 3584268",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222684803,
            "range": "± 15689570",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 411121112,
            "range": "± 10383994",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yingtong.lai@gmail.com",
            "name": "ying tong",
            "username": "therealyingtong"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "76b3f892a9d598923bbb5a747701fff44ae4c0ea",
          "message": "Merge pull request #774 from zhiqiangxu/fix_typo\n\nfix typo",
          "timestamp": "2023-05-24T01:32:29+08:00",
          "tree_id": "6ba08f4202beb6550d01cb602266adf29bc3435c",
          "url": "https://github.com/zcash/halo2/commit/76b3f892a9d598923bbb5a747701fff44ae4c0ea"
        },
        "date": 1684867518985,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 116859293,
            "range": "± 10735685",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 6736375,
            "range": "± 906339",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 215548725,
            "range": "± 8732331",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 8373218,
            "range": "± 811782",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 289446070,
            "range": "± 9317031",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 9093177,
            "range": "± 1069277",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 52400,
            "range": "± 2611",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 177752,
            "range": "± 6996",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 189767,
            "range": "± 7047",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 319865,
            "range": "± 9875",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 317293,
            "range": "± 17889",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 179704,
            "range": "± 8863",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 194153,
            "range": "± 9029",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 319103,
            "range": "± 28306",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 319837,
            "range": "± 15850",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 373655,
            "range": "± 14383",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 396031,
            "range": "± 16222",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 509426,
            "range": "± 18672",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 512888,
            "range": "± 21391",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 4097810,
            "range": "± 223206",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7821779,
            "range": "± 321594",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 13439351,
            "range": "± 158818",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 28428620,
            "range": "± 983636",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 54746680,
            "range": "± 1364591",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 109648268,
            "range": "± 1380475",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8619,
            "range": "± 912",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10649,
            "range": "± 1702",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 17933,
            "range": "± 2318",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 23577,
            "range": "± 2420",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33365,
            "range": "± 7321",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 54915,
            "range": "± 10097",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 131301,
            "range": "± 23625",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 277647,
            "range": "± 51706",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 561759,
            "range": "± 99265",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1019759,
            "range": "± 144086",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2092837,
            "range": "± 190188",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4668267,
            "range": "± 502510",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10109005,
            "range": "± 1118362",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21554671,
            "range": "± 2722817",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 45261455,
            "range": "± 2713210",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 98364420,
            "range": "± 3527242",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 35674,
            "range": "± 3670",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35433,
            "range": "± 3123",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 200091024,
            "range": "± 3809474",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 433936659,
            "range": "± 9319485",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 920937098,
            "range": "± 11679043",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1974936624,
            "range": "± 20977207",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4195141244,
            "range": "± 54476744",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 9046643692,
            "range": "± 55497709",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 18966006792,
            "range": "± 96158869",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 40248914711,
            "range": "± 363629475",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 86462460349,
            "range": "± 762622525",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 136230389,
            "range": "± 4972538",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 234618712,
            "range": "± 7026816",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 411606895,
            "range": "± 8690100",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 712706054,
            "range": "± 13759933",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1308175754,
            "range": "± 26378397",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2454554498,
            "range": "± 25562166",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4648603054,
            "range": "± 32206189",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 9038939055,
            "range": "± 54216621",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 17310333615,
            "range": "± 130187711",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6717006,
            "range": "± 820717",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10438757,
            "range": "± 1153751",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 17257395,
            "range": "± 1946523",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 29181372,
            "range": "± 2750926",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 49049923,
            "range": "± 5094580",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 86459555,
            "range": "± 4857052",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 157730836,
            "range": "± 9859280",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 293539131,
            "range": "± 14550422",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 535904725,
            "range": "± 11059317",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}