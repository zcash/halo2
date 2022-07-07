window.BENCHMARK_DATA = {
  "lastUpdate": 1657235789615,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "yingtong@z.cash",
            "name": "ying tong",
            "username": "therealyingtong"
          },
          "distinct": true,
          "id": "5af2bd3bd7779fdaa88f82d2b50a9fd1a46744bc",
          "message": "[Book] Rename \"polynomial degree bound\" to \"maximum constraint degree\".\nThis is because \"degree bound\" is often defined to be exclusive.\n\nSigned-off-by: Daira Hopwood <daira@jacaranda.org>",
          "timestamp": "2022-07-07T18:13:14-04:00",
          "tree_id": "102576b6338d3f2e451bbcb8b0194ee50f42757f",
          "url": "https://github.com/zcash/halo2/commit/5af2bd3bd7779fdaa88f82d2b50a9fd1a46744bc"
        },
        "date": 1657235783481,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 79263141,
            "range": "± 5969012",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3433618,
            "range": "± 64214",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 180560513,
            "range": "± 3358125",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4175657,
            "range": "± 76824",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 250354516,
            "range": "± 1058305",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4558507,
            "range": "± 92385",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47004,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 162980,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 178189,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 292372,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 292377,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 166397,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 181534,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 295890,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 295875,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 347882,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 363219,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 477668,
            "range": "± 223",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 477615,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3668686,
            "range": "± 2249",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6057876,
            "range": "± 19277",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10454784,
            "range": "± 11172",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22330890,
            "range": "± 150647",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41257764,
            "range": "± 263164",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 80951758,
            "range": "± 244955",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 6537,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 7814,
            "range": "± 530",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15647,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19518,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 28575,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 49370,
            "range": "± 1296",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 101491,
            "range": "± 1648",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 198631,
            "range": "± 8386",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 404983,
            "range": "± 8686",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 848358,
            "range": "± 5741",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1792434,
            "range": "± 27270",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3800918,
            "range": "± 26708",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8116919,
            "range": "± 137966",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17438725,
            "range": "± 123919",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 37435277,
            "range": "± 144957",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 80267768,
            "range": "± 475163",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 35055,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35147,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 176892167,
            "range": "± 1138884",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 380645272,
            "range": "± 443083",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 821567601,
            "range": "± 3358131",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1767213302,
            "range": "± 6625250",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3783424406,
            "range": "± 3884007",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8094002021,
            "range": "± 4364692",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17251732917,
            "range": "± 74278753",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36590912245,
            "range": "± 213830003",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77405327555,
            "range": "± 52433400",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 108600312,
            "range": "± 525637",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 187971527,
            "range": "± 3130999",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 331726875,
            "range": "± 1158402",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 604765533,
            "range": "± 1263891",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1125255406,
            "range": "± 4303262",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2125557248,
            "range": "± 5733862",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4039712416,
            "range": "± 11609777",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7781654094,
            "range": "± 31255978",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 14980135055,
            "range": "± 62867308",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5852052,
            "range": "± 87049",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9083229,
            "range": "± 214304",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14712276,
            "range": "± 297168",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24954829,
            "range": "± 556523",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42575414,
            "range": "± 895818",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76240670,
            "range": "± 1566028",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137195112,
            "range": "± 3352572",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 251076146,
            "range": "± 4911096",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 457810899,
            "range": "± 7966093",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}