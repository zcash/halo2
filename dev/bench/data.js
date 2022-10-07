window.BENCHMARK_DATA = {
  "lastUpdate": 1665156202498,
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
          "id": "c824785b7d61ec329de218d05cab20acfc671cea",
          "message": "Merge pull request #677 from zcash/therealyingtong-patch-1\n\n[doc] commitment::verifier: verify_proof does not have [-c]G term",
          "timestamp": "2022-10-07T15:11:07+01:00",
          "tree_id": "6fa6aac77e4eadee72dcf878b91638e8e1859633",
          "url": "https://github.com/zcash/halo2/commit/c824785b7d61ec329de218d05cab20acfc671cea"
        },
        "date": 1665155919456,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 74680419,
            "range": "± 7401959",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3597019,
            "range": "± 157303",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 162677411,
            "range": "± 2642135",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4391461,
            "range": "± 298869",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 225923796,
            "range": "± 5947919",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4886287,
            "range": "± 252009",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 48016,
            "range": "± 2182",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 171164,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 185819,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 301647,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 301694,
            "range": "± 876",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 174890,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 189582,
            "range": "± 2838",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 305272,
            "range": "± 537",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 305214,
            "range": "± 1757",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 365690,
            "range": "± 2281",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 380544,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 496016,
            "range": "± 792",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 496332,
            "range": "± 880",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3732542,
            "range": "± 2294",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7214390,
            "range": "± 90638",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12530658,
            "range": "± 38384",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 27850827,
            "range": "± 174619",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 52320546,
            "range": "± 79065",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 99978976,
            "range": "± 549968",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8947,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10125,
            "range": "± 350",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 19104,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 22524,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 32721,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 56812,
            "range": "± 2366",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 120192,
            "range": "± 4705",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 233801,
            "range": "± 10792",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 473553,
            "range": "± 22027",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1005058,
            "range": "± 52230",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2171807,
            "range": "± 74067",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4663566,
            "range": "± 161934",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10066531,
            "range": "± 344091",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 22333816,
            "range": "± 706700",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 48344001,
            "range": "± 1528990",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 107417008,
            "range": "± 4561287",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34186,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34395,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 193299245,
            "range": "± 12878021",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 409601056,
            "range": "± 11604569",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 889373553,
            "range": "± 10181408",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1899092576,
            "range": "± 27967576",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4081054558,
            "range": "± 61637638",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8477247074,
            "range": "± 73994001",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17896597840,
            "range": "± 196902232",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 37997154275,
            "range": "± 236964958",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 80213090683,
            "range": "± 404329551",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 115312120,
            "range": "± 1348605",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 200954085,
            "range": "± 5268700",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 358778668,
            "range": "± 9808433",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 665828647,
            "range": "± 15635138",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1242145151,
            "range": "± 12055231",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2269576944,
            "range": "± 36913978",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4340440498,
            "range": "± 65682258",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8377680137,
            "range": "± 60348654",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16065429743,
            "range": "± 193347429",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6200336,
            "range": "± 115882",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9878477,
            "range": "± 483569",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15594026,
            "range": "± 901518",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 27222431,
            "range": "± 1574177",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 47092518,
            "range": "± 1616434",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 80995125,
            "range": "± 3157747",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 143943442,
            "range": "± 5041738",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 267581793,
            "range": "± 8271212",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 485937158,
            "range": "± 11331435",
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
          "id": "21a79182ed439de886df99b59525c30539ba922e",
          "message": "Merge pull request #667 from Orbis-Tertius/fix-plonk-test\n\nDon't use `include_bytes!` for `plonk_api` test",
          "timestamp": "2022-10-07T15:14:38+01:00",
          "tree_id": "7a8766a5db97171e855a33b6c24fd49b64f2848a",
          "url": "https://github.com/zcash/halo2/commit/21a79182ed439de886df99b59525c30539ba922e"
        },
        "date": 1665156194823,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 76361863,
            "range": "± 6580116",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3778390,
            "range": "± 235374",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 169173334,
            "range": "± 8366467",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4796606,
            "range": "± 329921",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 228977100,
            "range": "± 7566742",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5241898,
            "range": "± 332000",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 46656,
            "range": "± 3407",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 171169,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 193721,
            "range": "± 13866",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 302970,
            "range": "± 496",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 302739,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 173847,
            "range": "± 3514",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 186558,
            "range": "± 2865",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 304140,
            "range": "± 5455",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 306370,
            "range": "± 4406",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 364232,
            "range": "± 7216",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 379846,
            "range": "± 3095",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 495585,
            "range": "± 8593",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 482070,
            "range": "± 14881",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3604164,
            "range": "± 91014",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7123833,
            "range": "± 111885",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12666988,
            "range": "± 62489",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 27478157,
            "range": "± 166026",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 51618486,
            "range": "± 501571",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 99874292,
            "range": "± 292464",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 9001,
            "range": "± 520",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10282,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18626,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 21948,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33110,
            "range": "± 770",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 57362,
            "range": "± 1744",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 123736,
            "range": "± 9713",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 242354,
            "range": "± 23318",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 477711,
            "range": "± 41726",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1061899,
            "range": "± 121951",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2154582,
            "range": "± 96043",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4741461,
            "range": "± 308587",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10437181,
            "range": "± 580224",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21881859,
            "range": "± 924050",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 47785862,
            "range": "± 1724505",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 116054478,
            "range": "± 6333693",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34265,
            "range": "± 3812",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34271,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 211420174,
            "range": "± 16320600",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 444677161,
            "range": "± 8460590",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 965542384,
            "range": "± 36471890",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 2017043162,
            "range": "± 101714034",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4277527969,
            "range": "± 165686189",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 9194132151,
            "range": "± 301861423",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 18325586450,
            "range": "± 471593061",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 38792972390,
            "range": "± 776597900",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 81495111462,
            "range": "± 308749922",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 138370000,
            "range": "± 13368394",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 228217407,
            "range": "± 5307619",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 388802403,
            "range": "± 14743623",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 716633669,
            "range": "± 19352647",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1353631749,
            "range": "± 20575748",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2561156437,
            "range": "± 132271077",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4292268687,
            "range": "± 193343294",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 9013273897,
            "range": "± 335287194",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16391836778,
            "range": "± 465895761",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6124611,
            "range": "± 160199",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9569798,
            "range": "± 210157",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15999391,
            "range": "± 1149249",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 26131827,
            "range": "± 1125166",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 45846752,
            "range": "± 2066872",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 82685652,
            "range": "± 4010851",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 142443331,
            "range": "± 2428880",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 269450981,
            "range": "± 10222776",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 480536474,
            "range": "± 7111428",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}