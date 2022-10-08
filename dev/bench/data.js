window.BENCHMARK_DATA = {
  "lastUpdate": 1665228352588,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0f7299c1165e98d1e26c73fabd6ab9429b64559e",
          "message": "Merge pull request #625 from zcash/region-query-instance\n\nhalo2_proofs: Introduce `RegionLayouter::instance_value` method.",
          "timestamp": "2022-10-08T11:18:46+01:00",
          "tree_id": "ffca7af06a8932ce4d740e7dd1d4ab39191d6433",
          "url": "https://github.com/zcash/halo2/commit/0f7299c1165e98d1e26c73fabd6ab9429b64559e"
        },
        "date": 1665228149810,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 66851455,
            "range": "± 5273851",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3455273,
            "range": "± 159856",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 141345039,
            "range": "± 3824245",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4200603,
            "range": "± 86623",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 199229396,
            "range": "± 1348321",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4641767,
            "range": "± 97497",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47886,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164197,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 179384,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 294480,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 294444,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 167639,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 182826,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 297825,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 297742,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 350801,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 365957,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 481037,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 481320,
            "range": "± 1380",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3490297,
            "range": "± 2628",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6065578,
            "range": "± 5044",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10435799,
            "range": "± 47318",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22807227,
            "range": "± 93121",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42893270,
            "range": "± 84799",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 81484933,
            "range": "± 163450",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7222,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9383,
            "range": "± 819",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15472,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20661,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 29336,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 49379,
            "range": "± 1474",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 102610,
            "range": "± 11605",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 199079,
            "range": "± 13112",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 405705,
            "range": "± 8447",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 850787,
            "range": "± 15564",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1791245,
            "range": "± 19209",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3791903,
            "range": "± 59088",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8093230,
            "range": "± 178791",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17460013,
            "range": "± 173640",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 37328118,
            "range": "± 323516",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 83851515,
            "range": "± 1834068",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34790,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34921,
            "range": "± 663",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 176869513,
            "range": "± 11078434",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 380813676,
            "range": "± 5419340",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 821364769,
            "range": "± 1741821",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1769159936,
            "range": "± 7876361",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3789216070,
            "range": "± 12359219",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8088032208,
            "range": "± 9463647",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17251322551,
            "range": "± 24536559",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36597606448,
            "range": "± 14843837",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77392349534,
            "range": "± 37462202",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 109560367,
            "range": "± 689728",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 189074552,
            "range": "± 1565105",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 336674530,
            "range": "± 2669747",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 608418262,
            "range": "± 1866554",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1124799123,
            "range": "± 2927586",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2134871869,
            "range": "± 61280969",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4058409271,
            "range": "± 11757718",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7821020935,
            "range": "± 16052633",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15078783707,
            "range": "± 41702199",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5862172,
            "range": "± 88234",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9120501,
            "range": "± 197195",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14885881,
            "range": "± 775073",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24862069,
            "range": "± 1135111",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42726037,
            "range": "± 333675",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76454241,
            "range": "± 2962343",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137258212,
            "range": "± 600079",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 252683207,
            "range": "± 9369649",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 458904051,
            "range": "± 3382060",
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
          "id": "de76fd48fc795c41daa6ab2233b95d1459b72c9c",
          "message": "Merge pull request #622 from zcash/patch-mockprover-query_instance\n\n[MockProver] Check for instance values in gate queries.",
          "timestamp": "2022-10-08T11:15:06+01:00",
          "tree_id": "9c8748762e3e9f090c03cdb29afb92ab67636780",
          "url": "https://github.com/zcash/halo2/commit/de76fd48fc795c41daa6ab2233b95d1459b72c9c"
        },
        "date": 1665228344002,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 75841023,
            "range": "± 3816837",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3746880,
            "range": "± 249853",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 162875279,
            "range": "± 8032335",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4724741,
            "range": "± 335101",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 225218194,
            "range": "± 7032582",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5176644,
            "range": "± 329392",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 50511,
            "range": "± 2043",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 178090,
            "range": "± 7585",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 191750,
            "range": "± 15800",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 313286,
            "range": "± 7933",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 313892,
            "range": "± 10323",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 179805,
            "range": "± 8160",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 195581,
            "range": "± 5990",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 317389,
            "range": "± 13019",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 317621,
            "range": "± 13964",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 382653,
            "range": "± 27597",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 389219,
            "range": "± 12808",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 517229,
            "range": "± 23918",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 511972,
            "range": "± 27389",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3836082,
            "range": "± 130980",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7618821,
            "range": "± 56567",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 13675453,
            "range": "± 113397",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 30567593,
            "range": "± 620484",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 57014952,
            "range": "± 625617",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 108419669,
            "range": "± 1421419",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8948,
            "range": "± 681",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10513,
            "range": "± 688",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 19422,
            "range": "± 1146",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 23387,
            "range": "± 1317",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33999,
            "range": "± 1432",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 58515,
            "range": "± 4845",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 121503,
            "range": "± 14082",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 242739,
            "range": "± 28889",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 492781,
            "range": "± 39824",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1030076,
            "range": "± 264401",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2273191,
            "range": "± 141140",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4915348,
            "range": "± 214414",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 11021687,
            "range": "± 1555732",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 22956623,
            "range": "± 1054070",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 49576155,
            "range": "± 1864594",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 111415378,
            "range": "± 4306195",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34934,
            "range": "± 1421",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34895,
            "range": "± 1659",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 200402007,
            "range": "± 14715380",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 413438663,
            "range": "± 5980796",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 900705127,
            "range": "± 70208622",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1936976295,
            "range": "± 18781697",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4143991480,
            "range": "± 48994313",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8967053402,
            "range": "± 171749272",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 18938307634,
            "range": "± 260757718",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 40157997237,
            "range": "± 243289317",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 84685244277,
            "range": "± 356799722",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 123566944,
            "range": "± 3279620",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 215615832,
            "range": "± 32629493",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 376195313,
            "range": "± 13646739",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 681473565,
            "range": "± 14527875",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1255315548,
            "range": "± 16684674",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2388595029,
            "range": "± 27693238",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4591928785,
            "range": "± 86933169",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8922690665,
            "range": "± 60495506",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 17172514331,
            "range": "± 109130480",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6493762,
            "range": "± 390270",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10120399,
            "range": "± 713158",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16767211,
            "range": "± 928785",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 27510751,
            "range": "± 1112823",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 47384368,
            "range": "± 2229134",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 84064888,
            "range": "± 3623572",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 152175056,
            "range": "± 5082674",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 282262992,
            "range": "± 7947666",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 523500359,
            "range": "± 16970620",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}