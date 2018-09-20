package pl.superhero;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.mongodb.MongoClient;
import com.mongodb.MongoClientURI;
import com.mongodb.client.MongoDatabase;
import io.vertx.core.Vertx;
import java.io.IOException;
import org.apache.http.HttpResponse;
import org.apache.http.client.fluent.Request;
import org.bson.Document;
import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import org.junit.*;
import static org.junit.Assert.assertThat;
import static pl.superhero.TestUtils.jsonEntity;
import static pl.superhero.TestUtils.readObject;

public class CreateHeroTest {

  private static ObjectNode heroes = readObject(CreateHeroTest.class.getResource("heroes.json"));
  private static MongoDatabase mongo;
  private static Vertx vertx;

  @BeforeClass
  public static void beforeClass() throws Exception {
    MongoClientURI uri = new MongoClientURI("mongodb://witoldsz:Q0PV6aCrggdTz1Kv"
        + "@superhero-shard-00-00-ty8ms.mongodb.net:27017,"
        + "superhero-shard-00-01-ty8ms.mongodb.net:27017,"
        + "superhero-shard-00-02-ty8ms.mongodb.net:27017"
        + "/testing"
        + "?ssl=true"
        + "&replicaSet=superhero-shard-0"
        + "&authSource=admin"
        + "&retryWrites=true");
    MongoClient mongoClient = new MongoClient(uri);
    mongo = mongoClient.getDatabase("testing");

    vertx = Vertx.vertx();
    TestUtils.<String>waitForCompletion(r -> vertx.deployVerticle(new MainVerticle(), r));
  }

  @AfterClass
  public static void afterClass() throws Exception {
    TestUtils.<Void>waitForCompletion(vertx::close);
  }

  @Before
  public void before() {
    mongo.getCollection("heroes").drop();
  }

  @Test
  public void should_create_hero() throws IOException {

    JsonNode batman = heroes.get("Batman");
    HttpResponse response = Request.Post("http://localhost:8080/heroes")
        .body(jsonEntity(batman))
        .execute()
        .returnResponse();
    System.out.println(org.apache.commons.io.IOUtils.toString(response.getEntity().getContent()));
    assertThat(response.getStatusLine().getStatusCode(), is(201));

    Document document = mongo.getCollection("heroes").find().first();
    assertThat(readObject(document.toJson()).without("_id"), equalTo(batman));
  }

}
