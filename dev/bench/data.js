window.BENCHMARK_DATA = {
  "lastUpdate": 1687891235489,
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
          "id": "eec65ead3bf5288162efa2bb2c23060db4258100",
          "message": "Merge pull request #646 from zcash/fix-pow5-pad\n\n`halo2_gadgets::poseidon`: Fix loading of padding words.",
          "timestamp": "2023-06-27T18:37:11+01:00",
          "tree_id": "f59f2d608ca6178233bd97d32a0c014eee644aec",
          "url": "https://github.com/zcash/halo2/commit/eec65ead3bf5288162efa2bb2c23060db4258100"
        },
        "date": 1687891000795,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 91827755,
            "range": "± 5979741",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4594248,
            "range": "± 450081",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 170315971,
            "range": "± 4611664",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5476059,
            "range": "± 441313",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 229630132,
            "range": "± 5951400",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7020270,
            "range": "± 620003",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 39606,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 140813,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153117,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 249894,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 249756,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 142546,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156029,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 252621,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 252651,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 300815,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 313247,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 409865,
            "range": "± 990",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 409940,
            "range": "± 1563",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3007467,
            "range": "± 2270",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5867060,
            "range": "± 14725",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10182639,
            "range": "± 47593",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22674247,
            "range": "± 67754",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42549601,
            "range": "± 150053",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 82952851,
            "range": "± 251512",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7266,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8640,
            "range": "± 917",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14863,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18584,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26130,
            "range": "± 2283",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 42452,
            "range": "± 5801",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 97093,
            "range": "± 16358",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 206086,
            "range": "± 35471",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 455958,
            "range": "± 60984",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 835640,
            "range": "± 78029",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1604818,
            "range": "± 161103",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3435005,
            "range": "± 361765",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7040069,
            "range": "± 261411",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15435787,
            "range": "± 370303",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 33808684,
            "range": "± 808035",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 77617441,
            "range": "± 824819",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28592,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28709,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153413895,
            "range": "± 6334846",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 327939360,
            "range": "± 1991154",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 711693610,
            "range": "± 4491109",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1513841223,
            "range": "± 6709166",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3246205699,
            "range": "± 8583645",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6934065257,
            "range": "± 17566616",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14767037944,
            "range": "± 30960049",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31367781362,
            "range": "± 67757970",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66282836773,
            "range": "± 86982170",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 103454470,
            "range": "± 4200699",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 181584609,
            "range": "± 6052678",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 314579919,
            "range": "± 9454340",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 556190329,
            "range": "± 8227530",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1008261300,
            "range": "± 12359732",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1892997078,
            "range": "± 12998455",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3613804221,
            "range": "± 22943593",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6946566648,
            "range": "± 24612799",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13319896935,
            "range": "± 50643732",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5058528,
            "range": "± 398625",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7931200,
            "range": "± 761461",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12767065,
            "range": "± 1388776",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21690662,
            "range": "± 1633373",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37496259,
            "range": "± 2666657",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67038312,
            "range": "± 4253442",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 119983632,
            "range": "± 2631279",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222538276,
            "range": "± 6853610",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 414271937,
            "range": "± 12789297",
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
          "id": "88ede7f1c6f07a61de02a9fd478028ecb72750c5",
          "message": "Merge pull request #775 from zcash/dependabot/github_actions/codecov/codecov-action-3.1.4\n\nBump codecov/codecov-action from 3.1.1 to 3.1.4",
          "timestamp": "2023-06-27T18:24:17+01:00",
          "tree_id": "345784e52a59a5492317a0909adc7e10d4deaf1d",
          "url": "https://github.com/zcash/halo2/commit/88ede7f1c6f07a61de02a9fd478028ecb72750c5"
        },
        "date": 1687891223269,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 108630556,
            "range": "± 12695502",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 6293984,
            "range": "± 792512",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 204805435,
            "range": "± 9435052",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 7666773,
            "range": "± 1069995",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 283295826,
            "range": "± 12467602",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 8621960,
            "range": "± 936532",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 51159,
            "range": "± 3114",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 167193,
            "range": "± 11489",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 178346,
            "range": "± 8775",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 296641,
            "range": "± 15915",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 299152,
            "range": "± 17959",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 171438,
            "range": "± 9642",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 180375,
            "range": "± 9793",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 301958,
            "range": "± 18849",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 312423,
            "range": "± 21197",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 358468,
            "range": "± 19212",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 380473,
            "range": "± 29034",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 507157,
            "range": "± 31964",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 507661,
            "range": "± 21702",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3943241,
            "range": "± 159186",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7963699,
            "range": "± 195271",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 14361174,
            "range": "± 462925",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 31333986,
            "range": "± 1123080",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 56701166,
            "range": "± 1637578",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 107903373,
            "range": "± 3323577",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8555,
            "range": "± 1678",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10535,
            "range": "± 1787",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 19668,
            "range": "± 3034",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 24559,
            "range": "± 4992",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33752,
            "range": "± 6878",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 55891,
            "range": "± 10786",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 140024,
            "range": "± 25012",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 285340,
            "range": "± 55890",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 561622,
            "range": "± 83052",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1011871,
            "range": "± 133379",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2047808,
            "range": "± 161631",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4493901,
            "range": "± 532604",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9676241,
            "range": "± 1081031",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 22079705,
            "range": "± 2653781",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 45583866,
            "range": "± 3024071",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 99737174,
            "range": "± 4213277",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 35388,
            "range": "± 2179",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35232,
            "range": "± 1607",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 196225184,
            "range": "± 11797291",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 427668476,
            "range": "± 7995278",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 917157500,
            "range": "± 16582114",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1949115249,
            "range": "± 29008840",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4142094325,
            "range": "± 26784913",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8829972346,
            "range": "± 39183290",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 18798324563,
            "range": "± 150456946",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 39861034362,
            "range": "± 238819189",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 85192197323,
            "range": "± 1302412095",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 139525301,
            "range": "± 4866426",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 240643754,
            "range": "± 7442500",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 412138145,
            "range": "± 12684215",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 734470576,
            "range": "± 20885337",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1316493741,
            "range": "± 33854157",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2488430865,
            "range": "± 37242015",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4718424091,
            "range": "± 40487357",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 9070406292,
            "range": "± 109064550",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 17488176299,
            "range": "± 90847700",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6986231,
            "range": "± 884154",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10805764,
            "range": "± 1365107",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16957046,
            "range": "± 2060873",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 28875740,
            "range": "± 3245705",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 50823882,
            "range": "± 5548602",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 89378690,
            "range": "± 6778843",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 160268629,
            "range": "± 7875533",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 295574640,
            "range": "± 22809356",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 546243468,
            "range": "± 14211573",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}