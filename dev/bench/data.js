window.BENCHMARK_DATA = {
  "lastUpdate": 1703028755495,
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
          "id": "c05547a70f452b525474861cc2337aefeb342adb",
          "message": "Merge pull request #782 from TomTaehoonKim/fix/typo\n\nFix typo",
          "timestamp": "2023-06-27T18:59:16+01:00",
          "tree_id": "5721a21b49641106a383a756fdaae4b15cdd2615",
          "url": "https://github.com/zcash/halo2/commit/c05547a70f452b525474861cc2337aefeb342adb"
        },
        "date": 1687892881829,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 105757196,
            "range": "± 8834795",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 5960744,
            "range": "± 609126",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 203266853,
            "range": "± 6961041",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6665452,
            "range": "± 845510",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 268913400,
            "range": "± 6314493",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7918564,
            "range": "± 934303",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 46505,
            "range": "± 1388",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 168715,
            "range": "± 1831",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 183264,
            "range": "± 1649",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 298635,
            "range": "± 3641",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 299316,
            "range": "± 3087",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 169087,
            "range": "± 3947",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 179437,
            "range": "± 5820",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 288399,
            "range": "± 9300",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 295686,
            "range": "± 6567",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 356790,
            "range": "± 6009",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 366092,
            "range": "± 8989",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 485877,
            "range": "± 12864",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 485462,
            "range": "± 12724",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3568128,
            "range": "± 28963",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6965754,
            "range": "± 28116",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12047631,
            "range": "± 73195",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 25386092,
            "range": "± 468094",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 47273946,
            "range": "± 677892",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 92628787,
            "range": "± 1631671",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8424,
            "range": "± 1020",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10197,
            "range": "± 2533",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16768,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 22594,
            "range": "± 787",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 31676,
            "range": "± 3901",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 53154,
            "range": "± 8876",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 119365,
            "range": "± 16006",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 247157,
            "range": "± 43825",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 520803,
            "range": "± 71770",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 954730,
            "range": "± 104438",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1821476,
            "range": "± 110023",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3952337,
            "range": "± 355867",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8438914,
            "range": "± 618393",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 18491966,
            "range": "± 1325254",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 40140386,
            "range": "± 1816682",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 90435063,
            "range": "± 3420978",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34290,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34433,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 175594417,
            "range": "± 1967755",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 384900021,
            "range": "± 8120577",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 825528271,
            "range": "± 10080238",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1760865346,
            "range": "± 27357003",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3785147635,
            "range": "± 40189945",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8221736987,
            "range": "± 58957409",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17712409571,
            "range": "± 115033205",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 37488796607,
            "range": "± 374413787",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 79482344189,
            "range": "± 231250291",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 121592731,
            "range": "± 5681590",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 213974085,
            "range": "± 10024284",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 369821899,
            "range": "± 7450101",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 662958225,
            "range": "± 13559457",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1204375444,
            "range": "± 19150692",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2245517486,
            "range": "± 33204312",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4355628492,
            "range": "± 42674054",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8316883403,
            "range": "± 131623508",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16003512752,
            "range": "± 158712505",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6162849,
            "range": "± 623305",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9524544,
            "range": "± 787551",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15869713,
            "range": "± 1732586",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 25996295,
            "range": "± 1867292",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 44892821,
            "range": "± 3991732",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 82231081,
            "range": "± 4696226",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 156196325,
            "range": "± 8652573",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 265470284,
            "range": "± 14846814",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 493767093,
            "range": "± 21165139",
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
          "id": "f9838c127ec9c14f6f323e0cfdc0c1392594d37f",
          "message": "Merge pull request #788 from zcash/fix-msrv-1.60\n\nhalo2_proofs: Pin dev-dependencies to versions compatible with MSRV",
          "timestamp": "2023-07-30T15:26:56+01:00",
          "tree_id": "448b5820c09267ba06679bb3a89b8d4de0939e64",
          "url": "https://github.com/zcash/halo2/commit/f9838c127ec9c14f6f323e0cfdc0c1392594d37f"
        },
        "date": 1690730774337,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 90518191,
            "range": "± 8340744",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4628487,
            "range": "± 405593",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 169766754,
            "range": "± 5716707",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5461320,
            "range": "± 529360",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 227835624,
            "range": "± 7232166",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6886872,
            "range": "± 724281",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38657,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 142700,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 155000,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 252202,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 252487,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 145657,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 158063,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 255332,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 255243,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 304733,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 317215,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 414443,
            "range": "± 5990",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 414423,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3042807,
            "range": "± 5725",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5879178,
            "range": "± 14654",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10353637,
            "range": "± 38997",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22834973,
            "range": "± 277332",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43031195,
            "range": "± 258650",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 84345003,
            "range": "± 266512",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7421,
            "range": "± 550",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8876,
            "range": "± 1162",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15384,
            "range": "± 460",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20076,
            "range": "± 406",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26516,
            "range": "± 2524",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 43220,
            "range": "± 5670",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 98440,
            "range": "± 13661",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 204109,
            "range": "± 34826",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 454676,
            "range": "± 56442",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 851836,
            "range": "± 103077",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1633995,
            "range": "± 167881",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3207481,
            "range": "± 307592",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7047364,
            "range": "± 348107",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15491572,
            "range": "± 510649",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 34065893,
            "range": "± 489572",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 79148872,
            "range": "± 1256254",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28494,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28644,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153678314,
            "range": "± 2134899",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 326084440,
            "range": "± 1267462",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 717791548,
            "range": "± 9598043",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1511663331,
            "range": "± 9836462",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3244788282,
            "range": "± 6266234",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6924102594,
            "range": "± 11884180",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14757984034,
            "range": "± 16170040",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31360315440,
            "range": "± 25522755",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66212601294,
            "range": "± 55219169",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 104120013,
            "range": "± 2912976",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 179676923,
            "range": "± 4010586",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 312112066,
            "range": "± 9287529",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 554563210,
            "range": "± 7510004",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1007747114,
            "range": "± 14639163",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1882335928,
            "range": "± 14378596",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3579649683,
            "range": "± 12616958",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6950775775,
            "range": "± 21961050",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13316962016,
            "range": "± 96621430",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5147336,
            "range": "± 16937",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7968726,
            "range": "± 462049",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12724327,
            "range": "± 647261",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21638427,
            "range": "± 1379831",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37361019,
            "range": "± 2797418",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66230453,
            "range": "± 4509532",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 119542853,
            "range": "± 5727530",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 221708828,
            "range": "± 18187854",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 412199673,
            "range": "± 6377788",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira Emma Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7fd2ce259ec3d0b0e3ede3fa20e4cdcfc395efc9",
          "message": "Merge pull request #805 from zcash/check-in-lockfile\n\nAdd `Cargo.lock` to repository",
          "timestamp": "2023-11-29T21:52:01Z",
          "tree_id": "76a058cdb3e6c6aee9ec313880b5924b830526c7",
          "url": "https://github.com/zcash/halo2/commit/7fd2ce259ec3d0b0e3ede3fa20e4cdcfc395efc9"
        },
        "date": 1701297674691,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71524039,
            "range": "± 2322664",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4055991,
            "range": "± 94388",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136493524,
            "range": "± 4463301",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4587858,
            "range": "± 152257",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 188053375,
            "range": "± 1763037",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4989145,
            "range": "± 112380",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31148,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133428,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 145756,
            "range": "± 490",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 235747,
            "range": "± 6714",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 235845,
            "range": "± 5820",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 136149,
            "range": "± 3102",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148458,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 238471,
            "range": "± 1136",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 238387,
            "range": "± 7031",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 285187,
            "range": "± 928",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 296778,
            "range": "± 1151",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 387577,
            "range": "± 865",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 387444,
            "range": "± 1815",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2846756,
            "range": "± 38345",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 4685056,
            "range": "± 6195",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 8478702,
            "range": "± 73386",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 18107850,
            "range": "± 52953",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 33238425,
            "range": "± 99730",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 63261410,
            "range": "± 764212",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7906,
            "range": "± 1973",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8428,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14184,
            "range": "± 884",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18597,
            "range": "± 1127",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26767,
            "range": "± 1512",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 43729,
            "range": "± 3050",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 91509,
            "range": "± 1754",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 163357,
            "range": "± 3367",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 296172,
            "range": "± 7612",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 574252,
            "range": "± 15295",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1154971,
            "range": "± 27876",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 2382815,
            "range": "± 63803",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 5055151,
            "range": "± 99144",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 10763998,
            "range": "± 229630",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 22475827,
            "range": "± 656820",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 50080570,
            "range": "± 1278591",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 29176,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 29261,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 130484029,
            "range": "± 2861233",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 271415201,
            "range": "± 5887964",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 557388068,
            "range": "± 16881445",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1210030353,
            "range": "± 26482961",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 2577944545,
            "range": "± 68651712",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 5518402972,
            "range": "± 172617354",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 11319227221,
            "range": "± 217835681",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 24163282337,
            "range": "± 412375254",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 51346356793,
            "range": "± 617634251",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 85344499,
            "range": "± 959875",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 139982468,
            "range": "± 2304701",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 238119862,
            "range": "± 7144852",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 418123508,
            "range": "± 7111948",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 763395424,
            "range": "± 11482546",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1411762296,
            "range": "± 28414014",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 2663665301,
            "range": "± 22472190",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 5049260495,
            "range": "± 36764006",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 9814122777,
            "range": "± 40025535",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 4593339,
            "range": "± 262123",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 6623489,
            "range": "± 205767",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 10634982,
            "range": "± 197131",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 17494140,
            "range": "± 383873",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 29961365,
            "range": "± 497966",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 51667314,
            "range": "± 3561914",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 92362822,
            "range": "± 849190",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 166331806,
            "range": "± 4553581",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 309232676,
            "range": "± 11806779",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira Emma Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ae52332c776a44e5165c4e2bda53b5c2f6e087ce",
          "message": "Merge pull request #802 from daira/book-remove-makefile\n\nRemove obsolete book `Makefile` and `edithtml.sh`",
          "timestamp": "2023-12-19T22:46:19Z",
          "tree_id": "3fb596ffc87a3d33502efcc4f0fe426dd6365f27",
          "url": "https://github.com/zcash/halo2/commit/ae52332c776a44e5165c4e2bda53b5c2f6e087ce"
        },
        "date": 1703028747531,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 71782451,
            "range": "± 1021123",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4055317,
            "range": "± 191834",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 136460845,
            "range": "± 2801716",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4606532,
            "range": "± 135944",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 187883676,
            "range": "± 2731769",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5013375,
            "range": "± 155247",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 31185,
            "range": "± 549",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 133061,
            "range": "± 573",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 146199,
            "range": "± 1294",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 234529,
            "range": "± 329",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 234497,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 135725,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 148801,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 237232,
            "range": "± 531",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 237192,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 284266,
            "range": "± 1484",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 297635,
            "range": "± 1619",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 385436,
            "range": "± 6294",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 385501,
            "range": "± 1915",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2862486,
            "range": "± 13869",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 4675764,
            "range": "± 8597",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 8071737,
            "range": "± 12255",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 17910784,
            "range": "± 127131",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 32132104,
            "range": "± 426161",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 62240614,
            "range": "± 161421",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 9073,
            "range": "± 1917",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8293,
            "range": "± 1055",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 13987,
            "range": "± 1126",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18318,
            "range": "± 1101",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26537,
            "range": "± 1799",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 44187,
            "range": "± 2133",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 90858,
            "range": "± 1936",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 163256,
            "range": "± 1320",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 296483,
            "range": "± 3875",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 573956,
            "range": "± 12550",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1155078,
            "range": "± 32935",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 2390687,
            "range": "± 59137",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 5049097,
            "range": "± 72498",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 10692795,
            "range": "± 157135",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 22038115,
            "range": "± 370058",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 48107664,
            "range": "± 1093987",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 29234,
            "range": "± 1395",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 29327,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 129822221,
            "range": "± 1866712",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 270482753,
            "range": "± 7517591",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 578039319,
            "range": "± 17860453",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1220982915,
            "range": "± 33802164",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 2602464840,
            "range": "± 59622048",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 5483896305,
            "range": "± 115597999",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 11572739715,
            "range": "± 272871672",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 24127444775,
            "range": "± 556283574",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 51560433318,
            "range": "± 694294621",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 84676067,
            "range": "± 1074192",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 139211266,
            "range": "± 1051084",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 237098689,
            "range": "± 2189106",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 419191996,
            "range": "± 2245405",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 760949468,
            "range": "± 17275358",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1408331884,
            "range": "± 4670909",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 2665467118,
            "range": "± 21494715",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 5068260728,
            "range": "± 19696402",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 9772061987,
            "range": "± 22107749",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 4554583,
            "range": "± 66611",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 6618534,
            "range": "± 185330",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 10607906,
            "range": "± 178026",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 17355649,
            "range": "± 354181",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 29961314,
            "range": "± 1799735",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 51685557,
            "range": "± 581735",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 92270489,
            "range": "± 1076133",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 166210219,
            "range": "± 4625349",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 309340672,
            "range": "± 1376298",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}