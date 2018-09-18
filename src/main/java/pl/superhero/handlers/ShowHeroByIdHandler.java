package pl.superhero.handlers;

import io.vertx.core.Handler;
import io.vertx.ext.web.api.RequestParameters;
import io.vertx.ext.web.RoutingContext;

public class ShowHeroByIdHandler implements Handler<RoutingContext> {

    public ShowHeroByIdHandler(){

    }

    @Override
    public void handle(RoutingContext routingContext) {
        RequestParameters params = routingContext.get("parsedParameters");
        // Handle showHeroById
        routingContext.response().setStatusCode(501).setStatusMessage("Not Implemented").end();
    }

}