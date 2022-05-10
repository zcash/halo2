window.BENCHMARK_DATA = {
  "lastUpdate": 1652225193851,
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
          "id": "50921f95f7d4edf48f79ffba3b892a983d91ed7f",
          "message": "Merge pull request #573 from zcash/str4d/chip-review\n\nChanges from `halo2_gadgets` review",
          "timestamp": "2022-05-10T22:54:45+01:00",
          "tree_id": "58f6bfa11eb3602650e308cf35988358a64b9ea4",
          "url": "https://github.com/zcash/halo2/commit/50921f95f7d4edf48f79ffba3b892a983d91ed7f"
        },
        "date": 1652223166445,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 74075734,
            "range": "± 3367510",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3268829,
            "range": "± 56486",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 173352773,
            "range": "± 6335467",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4791217,
            "range": "± 417237",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 220755346,
            "range": "± 3116930",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5924371,
            "range": "± 97665",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 32371,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 126422,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 136677,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 221788,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 221683,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 129204,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 139358,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 224404,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 224452,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 270281,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 279877,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 364485,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 364524,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2756672,
            "range": "± 1091",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5192395,
            "range": "± 3185",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 9012742,
            "range": "± 9279",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 20384021,
            "range": "± 605247",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 38490011,
            "range": "± 156527",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 75735267,
            "range": "± 284623",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7270,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8799,
            "range": "± 1236",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15311,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 16864,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 25751,
            "range": "± 961",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 44966,
            "range": "± 1093",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 93162,
            "range": "± 5521",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 185885,
            "range": "± 4531",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 383549,
            "range": "± 19207",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 798442,
            "range": "± 18363",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1669484,
            "range": "± 29741",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3581125,
            "range": "± 29520",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7947104,
            "range": "± 599778",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17379654,
            "range": "± 212521",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38402111,
            "range": "± 117621",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 87026970,
            "range": "± 753102",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 25132,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 25191,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 151282765,
            "range": "± 5800738",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 321619506,
            "range": "± 797111",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 692923498,
            "range": "± 4808126",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1492368216,
            "range": "± 11349742",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3199630355,
            "range": "± 24682983",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6836048946,
            "range": "± 37948241",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14571462527,
            "range": "± 86321149",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 30978968714,
            "range": "± 38249807",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 65623542564,
            "range": "± 917556697",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 94238015,
            "range": "± 638082",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 162874006,
            "range": "± 1091145",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 290144656,
            "range": "± 961301",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 531554862,
            "range": "± 2309781",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 996304333,
            "range": "± 6497555",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1883750863,
            "range": "± 4020574",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3574379008,
            "range": "± 4546050",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6920147039,
            "range": "± 11460708",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13323950514,
            "range": "± 27603663",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5044334,
            "range": "± 22078",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7803652,
            "range": "± 20505",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12805226,
            "range": "± 91711",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21809472,
            "range": "± 70814",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37319620,
            "range": "± 1355350",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66319934,
            "range": "± 438728",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 118661359,
            "range": "± 2392633",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 221119060,
            "range": "± 4542089",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 401234204,
            "range": "± 3999592",
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
          "id": "bf459804f66b322317f0cddb2d727a26378765df",
          "message": "Merge pull request #581 from zcash/release-0.1.0\n\nRelease `halo2_proofs` and `halo2_gadgets` 0.1.0",
          "timestamp": "2022-05-10T23:27:30+01:00",
          "tree_id": "0602c00c9ce9884a720c40a222a28da02e132de5",
          "url": "https://github.com/zcash/halo2/commit/bf459804f66b322317f0cddb2d727a26378765df"
        },
        "date": 1652225188930,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 74918003,
            "range": "± 4676644",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3365187,
            "range": "± 25160",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 175271442,
            "range": "± 1630230",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4987453,
            "range": "± 39713",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 240480631,
            "range": "± 1747315",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6024304,
            "range": "± 74027",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 37401,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 143259,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 154727,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 250755,
            "range": "± 1292",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 250713,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 146278,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 157798,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 253612,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 253511,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 305899,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 316829,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 412268,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 412277,
            "range": "± 834",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3289490,
            "range": "± 1505",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5917004,
            "range": "± 3094",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10276310,
            "range": "± 65497",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22464116,
            "range": "± 176172",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 44464731,
            "range": "± 502238",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 83990164,
            "range": "± 582181",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7710,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8816,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15424,
            "range": "± 1326",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18541,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27379,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 46751,
            "range": "± 1794",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 97203,
            "range": "± 7560",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 188813,
            "range": "± 8880",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 386984,
            "range": "± 35353",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 817281,
            "range": "± 30665",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1718938,
            "range": "± 52650",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3694260,
            "range": "± 54363",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8134349,
            "range": "± 178187",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 18021517,
            "range": "± 148058",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 40222464,
            "range": "± 577663",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 88974339,
            "range": "± 1013687",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28476,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28524,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153158782,
            "range": "± 2668003",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 327917525,
            "range": "± 1105069",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 707171484,
            "range": "± 1084687",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1521103765,
            "range": "± 7332519",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3256344524,
            "range": "± 4847224",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6954477372,
            "range": "± 10079008",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14814476593,
            "range": "± 22911048",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31498305586,
            "range": "± 187995078",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66693554872,
            "range": "± 140163622",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 96318538,
            "range": "± 1026124",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 166587945,
            "range": "± 1647263",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 295466748,
            "range": "± 1447908",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 538671568,
            "range": "± 1483613",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1007826901,
            "range": "± 4150926",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1906922907,
            "range": "± 5045310",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3618186514,
            "range": "± 17131757",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6993554050,
            "range": "± 117072618",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13449175350,
            "range": "± 35195677",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5130144,
            "range": "± 48634",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8007412,
            "range": "± 135780",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 13063987,
            "range": "± 108709",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 22093145,
            "range": "± 286220",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37897313,
            "range": "± 190936",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66767540,
            "range": "± 792307",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120760260,
            "range": "± 1351053",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222863592,
            "range": "± 10489249",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 404602846,
            "range": "± 1335092",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}