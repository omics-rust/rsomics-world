#!/usr/bin/env Rscript

script <- grep("^--file=", commandArgs(trailingOnly=FALSE), value=TRUE)
script_dir <- dirname(normalizePath(sub("^--file=", "", script)))
driver <- file.path(script_dir, "infercnv_subclusters_workflow.R")
if (file.exists(driver)) source(driver)
stopifnot(exists("subcluster_cut_record", mode="function"))

tree <- structure(list(merge=matrix(c(-1L, -3L, -2L, 1L), ncol=2L, byrow=TRUE),
                       height=c(0.1, 10), order=c(2L, 1L, 3L),
                       labels=c("a", "b", "c"), method="ward.D2",
                       dist.method="euclidean"), class="hclust")
mixed <- subcluster_cut_record(tree, 1:3, list(g_s1=c(2L, 1L), g_s2=3L), "pheight", 0.5)
stopifnot(mixed$cut_height == "5", identical(unlist(mixed$cut_labels), c(1L, 2L, 1L)),
          !mixed$matches_cutree_partition,
          identical(unlist(mixed$subclusters[[1L]]$cut_labels), c(2L, 1L)))
cat("Ward characterization distinguishes mixed traversal chunks from actual tree clusters\n")

valid <- subcluster_cut_record(tree, 1:3, list(g_s1=c(1L, 3L), g_s2=2L), "qnorm", 0.5)
small <- subcluster_cut_record(NULL, 1:2, list(g_s1=1:2), "pheight", 0.5)
stopifnot(valid$matches_cutree_partition, as.numeric(valid$cut_height) == 5.05,
          small$matches_cutree_partition, is.null(small$cut_height),
          identical(unlist(small$cut_labels), c(1L, 1L)))
cat("Ward characterization retains valid partitions and unclustered small groups\n")
