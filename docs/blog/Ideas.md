# Blog Ideas (try to keep to 1 min)

* Machine Inference != Machine Learning
    - Trend of using pretrained models
    - Many advantages
        - Implement into your product
        - Evaluate baseline accuracy, throughput, latency
    - Many issues
        - Cannot improve upon
        - Failure in the cases in the wild, what do you do?
        - May have been trained with improper license
    - Conclusion
        - We are working helping companies with this problem, contact us for access

* Stop Training on Static Datasets
    - Kaggle is a great place to grab data
    - The problem is, this data never changes
    - If you train on a static data set, and don't have the infrastructure to add to the dataset, modify it, retrain on a specific state, it can bite you later on down the road.
    - This is not the case in the real world, or inside your organization
    - At Oxen.ai we are working on solving this problem
    - Let's see how it works
        - Grab a dataset from Kaggle
            - When life gives you lemons
            - https://www.kaggle.com/datasets/yusufemir/lemon-quality-dataset
        - This isn't that many images to be honest...
            - What if we want to build this dataset up from 2,000 images to 10k? 100k?
            - All these images are on a similar white background, what if the lemon is on the floor? A granite counter? In a lemon factory? On a lemonade stand?
        - Index the initial images into Oxen
            - Show why you would not want to do this in git
        - Add more images
            - Make a branch, Greg's lemons
            - I took a picture of some lemons and put them in their proper folders
            - Push the branch
            - Pull the branch and look at the new images
            - Look good? Merge into main and push again
        - Conclusion: We do this for code all the time, why aren't we doing it for data? Let's clean up our data practices
    - What if we want to add more features? (New blog post on adding more features)

* Building a real world Computer Vision model for fitness applications
    - Back in 2019 I had the idea of "Computer Vision for Fitness"
    - I created a little demo here
        - (https://twitter.com/gregschoeninger/status/1145866472977137664?s=21)
    - I built the end to end app
        - https://twitter.com/gregschoeninger/status/1145867160096403456?s=20&t=gITu05WViFNn4VcFyv86Ag
    - I got help from a venture studio to get it out into the public
    - We got highlighted on twitter and became the app of the day
        - (https://twitter.com/AppStore/status/1393590809166028808?s=20&t=IjCbvGMoQ-Gi2lkHADL6pw)
        - (https://apps.apple.com/us/story/id1568601016)
    - But that's just the start of the journey
    - We started seeing failure cases in the wild

* Start with the Data
    - You should have your retraining pipeline ready to go when you deploy your app to prod
        - you do this for code, many do not for ML
    - Ideally you start with a model and dataset you can train on that data
    - Where do you get the data?
        - Cross link to tweets or other posts
        - Share with person that tweeted
    - How do you update the data?
    - What do you do if a model checkpoint is producing errors?
    - How do you even analyze those errors?
    - Conclusion
        - Create an oxen account to manage your data properly

* Your Current Eval Dataset sucks
    - Your current evaluation dataset may not be that great
        - Incorrectly tagged
        - Small images
        - Not representitive of actual use case
    - You should be finding error cases in the wild, iterating on, and fixing
    - Always a moving target
    - Need ability to go back to versions
    - Need ability to visualize false positives, true negatives, etc
        - Find me all the images where there was a wrist but I didn't tag
    - Conclusion
        - Need oxen to help manage evaluation dataset

* Human Performance Analysis
    - 4 key ML problems:
        - Classification (is there even a human in this image?)
        - Bounding box detection
        - Person Keypoints
        - Action classification based on keypoints
    - Conclusion
        - Link to 4 Oxen datasets


* Computer Vision Outcomes, Business Cases

    These are to spark the imagination, ping us with other ideas

        - So you want to start a dog park, AI beauty pagent, etc?
        - Point to other peoples works

    v1) 1 min bites of imagination
    v2) We link to a dataset in Oxen
    v3) We show training / eval on that dataset


    - Human Performance Analysis
        - Person Classification
        - Person Bounding Box
        - Person Keypoints
        - Person Action Classification
        - Person Action Prediction

    - Animal Identification, and Analysis
        - Tagging in the wild
        - Facial Recognition

    - Receipt Scanning
        - Image Classification
        - Image Bounding Box
        - Image OCR

    - Satellite Imagery
        - https://github.com/robmarkcole/satellite-image-deep-learning
        - Land Use/Cover
        - Vegitation, Crop Boundaries
        - Water Vs Land Segmentation, Flood Detection
        - Object Counting

    - Traffic / Street Analysis
        - Person Bounding Box
        - Car Bounding Box
        - Car Re-Identification
        - License Plate Identification
        - Person Re-Identification
    
    -  Autonomous Delivery
        - Drone flight path
        - Coco delivery

    - MRI Classification
        - 

    - Captcha To Gather Training Data
        - 

    - Chatbot
        - What are these components?
    
    - Product Review Analyzer
        - Sentiment Analysis
    
    - Copy Writer
        - Copy.ai
    
    - Pick the perfect NFL draft
    

* Human Performance Analysis
    - 4 key ML problems:
        - Classification (is there even a human in this image?)
        - Bounding box detection
        - Person Keypoints
        - Action classification based on keypoints
    - Conclusion
        - Link to 4 Oxen datasets



* Going from image to video in Human Performance Analysis
    - Naively running image models
    - Having some sort of history to take into account video
        - Performance issues, model export issues
    - Conclusion
        - Link to an Oxen dataset to get started

* Combining Pose Estimation Datasets
    - MSCoco 17 keypoints
    - AI challenger 14 keypoints
        - https://arxiv.org/abs/1711.06475
    - Crowdpose dataset
    - Leeds Sports dataset
        - http://sam.johnson.io/research/lsp.html
    - Need to combine to lowest common denominator
        - Leeds Sports 150x150 pixel and 14 keypoints
        - This can lead to performance hits
        - Might want to extend MSCoco to keep high quality images
    - Conclusion
        - Link to all these datasets

* Pose Dataset for your use case
    - Multi-human vs Single Pose
    - Pretrained on imagenet, some other subset?
    - Conclusion
        - Link to filtered down unsupervised or supervised datasets
        - Filter down dataset based on an image classifier
        - Call to improve/contribute/fork dataset for your use case

* Pose estimation evaluation technique (PCKh vs OKS)
    - Example code, based on Oxen format
    - oxen clone http://hub.oxen.ai/oxen/PoseEstimation
    - Conclusion
        - You should be constantly evaluating your models and understanding the metric

* Bounding box evaluation technique (IoU)
    - Example code, based on Oxen data format
    - oxen clone http://hub.oxen.ai/oxen/PersonBoundingBox
    - oxen checkout data branch

* Dataset licenses and what they mean?
    - Do research here

* Experiment: Image super resolution
    - Leeds sports is only 150x150
    - Can we upsample to 224x224?

* Experiment: Human image generation
    - Can we use stable diffusion + pose to generate new humans in same pose?


* Computer Vision Tasks (In terms of tech needed)
    - Image Classification
        - Hot Dog or Not?
        - Search/Filtering
        - Add Relevance
        - Topic Modeling/Clustering
        - Policy Enforcement
    - Object Detection
        - Bounding Box Around Object Type
            - Person, Animal, License Plate, Receipt, Product
        - Find all the people in this image
        - Crop subimage you are interested in
        - Damage and Defect Detection
        - Preduct Identification
        - Satelite Imagery
    - Object Keypoints
        - Human Pose Estimation
        - Face Keypoints
        - Animal joint keypoints
        - Hand Keypoints, Gestures
    - Object Segmentation
        - Pixel Level Segmentation of Objects
        - Crop background
    - Optical Character Recognition
        - License Plate Reading
        - Receipt Scanning
        - Product Identification
    - Prediction and Planning
        - Predict where the object you detected is going next
        - Predict where you should go next
        - Autonomous Vehicles
        - Delivery Robots
        - Delivery Drones