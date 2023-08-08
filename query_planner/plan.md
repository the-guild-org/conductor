```graphql
{
    reviews {
        id
        comment
        rating
        location {
            name
            description
        }
    }
}
```

1. REVIEWS Service
   1. fetch the nested fields
2. PARALLEL:
   1. for each review, fetch location

or:

3. REVIEWS Service
   1. fetch reviews with nested fields
4. SINGLE REQUEST:
   1. fetch all locations in one batch request


First, it recognizes that the reviews field is owned by the REVIEWS graph. Thus, it starts the plan by making a request to the REVIEWS service with the original query to fetch the id and rating of each review.

During this process, it also recognizes that location is a field that belongs to another service, LOCATIONS. However, it needs the id of each review's location to request this data, which is something the REVIEWS service should provide.

After the first step, it would have the list of ids for each location related to the reviews from the REVIEWS service.

For each unique id, it then makes a request to the LOCATIONS service to fetch the name and description fields for each location.

The results from both services are then merged together according to the original structure of the client's query to provide the final response.

The operations that are executed in parallel or sequentially depend on the data requirements and the directives applied. For example, if the location field had a @requires directive specifying that it needs certain fields from the REVIEWS service, then the query to the LOCATIONS service would be delayed until the REVIEWS service response is available.

Here's a pseudo-visualization of the query plan:

Query REVIEWS service: reviews { id rating location { id } }
Collect location ids from reviews.
For each unique location id, query LOCATIONS service: location(id: <id>) { name description }
Stitch results together into final form.