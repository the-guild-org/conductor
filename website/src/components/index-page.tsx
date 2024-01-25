import {
  FiAlertTriangle,
  FiArrowRightCircle,
  FiCloudLightning,
  FiCloudSnow,
  FiGlobe,
  FiLink,
  FiList,
  FiMoreHorizontal,
  FiShuffle,
  FiTarget,
  FiUserCheck,
} from 'react-icons/fi';
import { Feature } from './common/feature';
import { FlowManyChart } from './common/flow-many';
import { Highlights } from './common/highlight';
import { Ribbon } from './common/ribbon';
import { ExamplesView } from './examples-view';

export function IndexPage() {
  return (
    <>
      <div className="w-full overflow-x-hidden">
        <Ribbon>now on alpha</Ribbon>
        <div className="pt-[78px]">
          <h2 className="max-w-screen-[200px] mx-auto px-5 text-[32px] font-extrabold text-[#0f766e] lg:px-0 lg:text-center lg:text-5xl">
            A Fully-Featured, Open-Source
            <br />
            GraphQL Gateway for Your Project
          </h2>
        </div>
      </div>
      <div className="flex flex-col">
        <div className="w-full py-5">
          <div className="container mx-auto box-border flex flex-col px-6">
            <div className="mb-28 flex flex-col items-start lg:items-center lg:text-center">
              <div className="lg:px-8 lg:text-xl">
                <span className="font-medium lg:leading-8">
                  Conductor enriches your GraphQL runtime with powerful features such as caching,{' '}
                  <br className="hidden lg:block" />
                  rate-limiting, federation support and monitoring using a simple configuration
                  file.
                </span>
              </div>
              <div className="mt-8">
                <FlowManyChart />
              </div>
            </div>
            <Feature
              title="Simply Flexible"
              description={
                <div>
                  Conductor can handle many GraphQL sources, combine, transform and federate them.
                  <br className="hidden lg:block" />
                  You can also expose many GraphQL endpoints from a single instance, with different
                  schemas, variations and plugins.
                </div>
              }
            >
              <Highlights
                className="flex-col md:flex-col"
                items={[
                  {
                    title: 'Runs Everywhere',
                    description: (
                      <p className="text-gray-600 dark:text-gray-400">
                        Conductor runs on your own infrastructure. You can run it as a binary or in
                        a containerized environment like Kubernetes or Docker. You can also run
                        Conductor as a CloudFlare Worker (WASM-compiled).
                      </p>
                    ),
                    icon: <FiCloudSnow size={36} />,
                  },
                  {
                    title: 'Federated Execution',
                    description:
                      'We handle all the complexities under the hood to support Apollo Federation including query planning, steps execution, parallelization and response merging. (Fusion support is also on the way!)',
                    icon: <FiShuffle size={36} />,
                  },
                  {
                    title: 'Multiple Endpoints',
                    description:
                      'Expose the same or different GraphQL sources(s) on multiple endpoints: each endpoint can have its own flow and plugins.',
                    icon: <FiList size={36} />,
                  },
                  {
                    title: 'Custom Logic',
                    description:
                      "Aside from the set of plugins we offer, we've made it extremely easy to write your own custom plugins and conditional logic without the setup, and compliation headache. Using simple but powerful pre-compiled scripting with VRL (Vector Remap Language).",
                    icon: <FiLink size={36} />,
                  },
                ]}
              />
            </Feature>
            <Feature
              title="Unlimited Extensibility"
              flipped
              className="pb-0"
              description={
                <>
                  <div>
                    <strong>Enterprise-like features with zero effort at no cost.</strong> Choose
                    from our curated list of plugins, or develop and deploy your own.
                  </div>
                  <Highlights
                    className="mt-6 flex-col md:flex-col"
                    items={[
                      {
                        title: 'Response Caching',
                        description:
                          'Connect single or multiple caching stores, and simply plug-in caching to your GraphQL services. We support in-memory, redis and CloudFlare KV stores.',
                        icon: <FiCloudLightning size={36} />,
                      },
                      {
                        title: 'Security',
                        description:
                          'Built-in plugins for popular authentication flows (JWT with JWKS). Also, hardening plugins like rate-limit are available.',
                        icon: <FiUserCheck size={36} />,
                      },
                      {
                        title: 'Monitoring',
                        description:
                          'Monitor your service with built-in support for telemetry (OpenTelemetry, Jaeger, DataDog).',
                        icon: <FiAlertTriangle size={36} />,
                      },
                      // {
                      //   title: 'GraphQL to REST',
                      //   description: (
                      //     <p className="text-gray-600 dark:text-gray-400">
                      //       Expose any GraphQL schemas as REST service, powered by{' '}
                      //       <a href="https://www.the-guild.dev/graphql/sofa-api" target="_blank">
                      //         <strong>SOFA</strong>
                      //       </a>
                      //     </p>
                      //   ),
                      //   icon: <FiArrowRightCircle size={36} />,
                      // },
                      {
                        title: 'and many more',
                        icon: <FiMoreHorizontal size={36} />,
                      },
                    ]}
                  />
                </>
              }
              side={<ExamplesView />}
            />

            <div className="mb-8">
              <h1 className="mb-7 text-[32px] font-extrabold lg:text-5xl">FAQ</h1>

              <h3 className="mb-2 text-[20px] font-semibold lg:text-3xl">
                Is Conductor a drop-in replacement for Federation support in Apollo Router?
              </h3>
              <p>
                The end goal for Conductor is to be a drop-in replacement for Apollo Router, but as
                of now, we can't say it can be a drop-in replacement, due to the unavailablity of a
                spec for Apollo Federation and the strict licenses that limits the usage and
                inspiration of Apollo's code. We had to reverse engineer it from the ground up, and
                we've marked our launch as Alpha for this particular reason, to get people to try it
                out, and get an idea of our coverage and quickly iterate upon early users feedback
                to get it to our end goal.
              </p>
              <h3 className="mb-2 mt-4 text-[20px] font-semibold lg:text-3xl">
                How can we assure Conductor will remain MIT and Free?
              </h3>
              <p>
                The Guild's financial model stands in an extremely unique and strong position, we're
                self-sustained since day one, we've never took any external investments, our focus
                and passion as a group is building Open Source that lasts forever. To be able to
                achieve our goals, we've always stayed away from Venture Capitalist to have full
                control and freedom to persue our goals and values. We self-sustain ourselves from
                our SaaS products, consulting, and training. We have no financial pressure that
                would force us to change the license of Conductor or make it paid any time soon and
                further.
              </p>
              <h3 className="mb-2 mt-4 text-[20px] font-semibold lg:text-3xl">
                How can we assure you'll stay around on the long run?
              </h3>
              <p>
                The Guild has been around for more than a decade, maintaining and contributing to
                most of the leading GraphQL libraries for many years, with GraphQL Code Generator
                being one of our popular actively maintained and improved projects since 2015. As
                we've mentioned, we have no financial pressure to stop doing what we do anytime
                soon.
              </p>
              <h3 className="mb-2 mt-4 text-[20px] font-semibold lg:text-3xl">
                How is support for Enterprise-like features as in Apollo Router?
              </h3>
              <p>
                Conductor offers all of the Enterprise-like features you expect from other gateways
                at no cost. We have Authentication, Caching, and Open Telementry plugins that are
                extremely easy to use.
              </p>
              <h3 className="mb-2 mt-4 text-[20px] font-semibold lg:text-3xl">
                Can I contribute to Conductor?
              </h3>
              <p>
                Of course! Our roadmap is{' '}
                <a
                  className="text-teal-600"
                  target="_blank"
                  href="https://github.com/the-guild-org/conductor/issues/76"
                >
                  publically published
                </a>
                , you can open new issues for requested features or report bugs, you can tell us if
                we need to reprioritize our roadmap, and open pull requests.
              </p>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
