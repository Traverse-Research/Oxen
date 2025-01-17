# Competitors

- DVC
- Weights & Biases
- git + git lfs (hugging face datasets)
- perforce




# Why Oxen?

- Separate data and code
    - Different datasets can be used with the same codebase
    - Share datasets without sharing code
    - Easy to accidentally commit data to the codebase
- Datasets have many files, or long CSVs that Oxen works well with natively

# git lfs Comparison

CelebA 10K

```shell
$ git init .

$ git lfs track examples/

$ git add .gitattributes

$ git add examples

$ git commit -m "adding examples"
```

`git add`

```shell
time git add examples

________________________________________________________
Executed in    5.88 secs    fish           external
   usr time    1.37 secs    0.09 millis    1.37 secs
   sys time    3.34 secs    3.32 millis    3.34 secs
```

`git commit -m "adding examples"`

```shell
time commit -m "adding examples"

________________________________________________________
Executed in    1.83 secs      fish           external
   usr time  173.26 millis    0.28 millis  172.98 millis
   sys time  393.70 millis    1.54 millis  392.16 millis
```

# DVC Comparison

For DVC you put your data within a git repository. There is also an option to do without an existing version control system...TODO look into this.

```shell
$ git init .

$ dvc init

$ git commit -m "Initialize DVC"

$ dvc add img_align_celeba/

$ dvc remote add -f -d storage s3://dvc-test-celeba

$ dvc push
```

The `dvc add` command computes all the hashes of the data, as well as copies it all over to a directory as far as I can tell. It takes awhile....To be exact:

```shell
$ time dvc add img_align_celeba/
________________________________________________________
Executed in  354.10 secs    fish           external
   usr time  179.23 secs    0.31 millis  179.23 secs
   sys time  147.19 secs    1.68 millis  147.19 secs
```


```shell
time dvc status
Data and pipelines are up to date.                                    

________________________________________________________
Executed in    9.72 secs    fish           external
   usr time    2.68 secs    0.32 millis    2.68 secs
   sys time    5.79 secs    1.91 millis    5.79 secs
```


## DVC Push on smaller dataset

12438 files pushed (should have been 12500?)

total time for DogsVsCats test dataset with 12500 images was `500.72 secs` compared to our `201.64 secs`

# Wandb Comparison

>>> wandb.log_artifact('img_align_celeba/', name='celeba_images', type='images')
wandb: Adding directory to artifact (./img_align_celeba)... 
Done. 70.1s

# Oxen

DogsVsCats 

Push test 12500 images
   
* local no docker

________________________________________________________
Executed in   11.87 secs    fish           external
   usr time   15.77 secs    0.25 millis   15.77 secs
   sys time    4.39 secs    1.68 millis    4.39 secs


* local with docker-compose+traefik with 4 oxen-servers

________________________________________________________
Executed in   80.48 secs    fish           external
   usr time   26.84 secs    0.24 millis   26.84 secs
   sys time    7.49 secs    1.85 millis    7.49 secs


* local with 1 docker container

________________________________________________________
Executed in  105.13 secs    fish           external
   usr time   29.57 secs    0.29 millis   29.57 secs
   sys time    7.89 secs    1.80 millis    7.89 secs


* remote bare metal t2.medium at apartment

577.5 Mbps download
24.4 Mbps upload

________________________________________________________
Executed in  202.66 secs    fish           external
   usr time   68.02 secs    0.25 millis   68.02 secs
   sys time   26.07 secs    1.72 millis   26.07 secs

* remote bare metal i4i.2xlarge at apartment

577.5 Mbps download
24.4 Mbps upload

________________________________________________________
Executed in  199.70 secs    fish           external
   usr time   39.72 secs    0.30 millis   39.72 secs
   sys time   15.02 secs    1.69 millis   15.02 secs

* remote docker i4i.2xlarge at apartment

________________________________________________________
Executed in  201.64 secs    fish           external
   usr time   69.33 secs    0.24 millis   69.33 secs
   sys time   26.74 secs    1.57 millis   26.74 secs



CelebA 10k

* push local (all threads) - 9.97 secs

* pull AWS (all threads) - 174.49 secs

* push local (chunks) - 8.22 secs

* pull local (chunks) - 8.79 secs

Celeb 100k

* pull AWS (all theads) - 71.75 mins

* push local (compress all in one) - 218.88 sec

* push local (chunk and compress 128 chunks) - 126.69 sec

* push local (chunk and compress 1024 chunks) - 65.94 sec

* pull local (chunk and compress 1024 chunks) - 94.58 secs


# Rsync

rsync MSCoco/train2017 on local network (9.8 GB)

22.71 mins


